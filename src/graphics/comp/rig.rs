use crate::graphics::comp::parse::{MRFParser, ParsedKeyframe, Path, PathValue};
use crate::math::vec::Vec2;
use crate::rendering::Transform;
use parking_lot::Mutex;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::Arc;
use mvutils::unsafe_utils::Unsafe;
use mvutils::utils::Time;

#[derive(Debug)]
pub struct Part {
    index: usize,
    transform: Transform,
    prev_transform: Transform,
    bounds: Vec2,
    children: Vec<Arc<Mutex<Part>>>,
}

#[derive(Debug)]
pub struct Keyframe {
    percentage: f32,
    changes: HashMap<usize, Vec<(Path, PathValue)>>,
}

#[derive(Debug)]
pub struct RigAnimation {
    keyframes: Vec<Keyframe>,
    pre: Keyframe,
}

pub struct Rig {
    parts: Vec<Arc<Mutex<Part>>>,
    animations: HashMap<String, RigAnimation>,
    part_lookup: Option<Vec<Arc<Mutex<Part>>>>,
    current_time: f32,
    current_animations: HashMap<String, (u32, u128)>, //duration, start_time
}

impl Rig {
    pub fn from_parsed(parser: MRFParser) -> Self {
        let mut part_map: HashMap<String, Arc<Mutex<Part>>> = parser.parts
            .iter()
            .map(|p| (p.name.clone(), Arc::new(Mutex::new(Part {
                index: p.index,
                transform: Transform::new(),
                prev_transform: Transform::new(),
                bounds: p.bounds,
                children: vec![],
            })))).collect();

        let mut animations = HashMap::new();
        for parsed_anim in &parser.anims {
            let pre = Self::process_parsed_keyframe(&parsed_anim.pre, 0.0, &part_map);
            let mut grouped_keyframes: HashMap<(u64, i16, i8), (Vec<&ParsedKeyframe>, f32)> = HashMap::new();

            for (perc, keyframe) in &parsed_anim.keyframes {
                let dec = mvutils::utils::integer_decode(*perc as f64);
                grouped_keyframes.entry(dec).or_insert((vec![], *perc)).0.push(keyframe);
            }

            let mut keyframes = Vec::new();
            for (keyframe_group, percentage) in grouped_keyframes.into_values() {
                let kf = Self::process_parsed_keyframe2(keyframe_group, percentage, &part_map);
                keyframes.push(kf);
            }

            keyframes.sort_by(|k1, k2| k1.percentage.partial_cmp(&k2.percentage).unwrap_or(Ordering::Equal));

            animations.insert(parsed_anim.name.clone(), RigAnimation { keyframes, pre });
        }

        let mut child_map: HashMap<String, Vec<Arc<Mutex<Part>>>> = HashMap::new();

        for part in &parser.parts {
            if let Some(parent_name) = &part.parent {
                if let Some(child) = part_map.remove(&part.name) {
                    child_map.entry(parent_name.clone()).or_default().push(child);
                }
            }
        }

        for (parent_name, mut children) in child_map {
            if let Some(parent) = part_map.get_mut(&parent_name) {
                parent.lock().children.append(&mut children);
            }
        }

        let parts: Vec<Arc<Mutex<Part>>> = part_map.into_values().collect();

        let mut this = Self {
            parts,
            animations,
            part_lookup: None,
            current_time: 0.0,
            current_animations: HashMap::new(),
        };

        let mut part_lookup: Vec<Arc<Mutex<Part>>> = Vec::new();
        unsafe {
            for part in &this.parts {
                Self::flatten_part_hierarchy(part.clone(), &mut part_lookup);
            }
        }

        this.part_lookup = Some(part_lookup);
        this
    }

    unsafe fn flatten_part_hierarchy<'b>(part: Arc<Mutex<Part>>, lookup: &mut Vec<Arc<Mutex<Part>>>) {
        lookup.push(part.clone());
        let mut lock = part.lock();
        for child in &lock.children {
            Self::flatten_part_hierarchy(child.clone(), lookup);
        }
    }

    fn process_parsed_keyframe(parsed_keyframes: &[ParsedKeyframe], at: f32, map: &HashMap<String, Arc<Mutex<Part>>>) -> Keyframe {
        let changes: Vec<(usize, Path, PathValue)> = parsed_keyframes.iter()
            .filter_map(|kf| map.get(&kf.target).map(|target| (target.lock().index, kf.path.clone(), kf.value.clone())))
            .collect();

        let mut changes_map: HashMap<usize, Vec<(Path, PathValue)>> = HashMap::new();
        for (target, path, value) in changes {
            changes_map.entry(target).or_insert_with(Vec::new).push((path, value));
        }

        Keyframe { percentage: at, changes: changes_map }
    }

    fn process_parsed_keyframe2(parsed_keyframes: Vec<&ParsedKeyframe>, at: f32, map: &HashMap<String, Arc<Mutex<Part>>>) -> Keyframe {
        let changes: Vec<(usize, Path, PathValue)> = parsed_keyframes.iter()
            .filter_map(|kf| map.get(&kf.target).map(|target| (target.lock().index, kf.path.clone(), kf.value.clone())))
            .collect();

        let mut changes_map: HashMap<usize, Vec<(Path, PathValue)>> = HashMap::new();
        for (target, path, value) in changes {
            changes_map.entry(target).or_insert_with(Vec::new).push((path, value));
        }

        Keyframe { percentage: at, changes: changes_map }
    }

    pub fn get_part_transform(&self, part: usize) -> Option<Transform> {
        self.part_lookup.as_ref().and_then(|lookup| {
            lookup.get(part).map(|part| part.lock().transform.clone())
        })
    }

    pub fn start_animation(&mut self, animation: &str, duration: u32) {
        self.current_time = 0.0;
        self.current_animations.insert(animation.to_string(), (duration, u128::time_millis()));

        let this = unsafe { Unsafe::cast_mut_static(self) };

        if let Some(anim) = self.animations.get(animation) {
            this.apply_keyframe(&anim.pre);
            if let Some(first) = anim.keyframes.first() {
                this.apply_keyframe(first);
            }
        }
    }

    pub fn tick(&mut self) {
        let this = unsafe { Unsafe::cast_mut_static(self) };
        let mut to_remove = Vec::new();
        for (anim_name, (duration, start_time)) in self.current_animations.iter() {
            if let Some(anim) = self.animations.get(anim_name) {
                let progress = (u128::time_millis() - start_time) as f32 / *duration as f32;
                if progress > 1.0 {
                    to_remove.push(anim_name);
                    continue;
                }
                let mut i = 0;
                for (idx, keyframe) in anim.keyframes.iter().enumerate() {
                    if progress < keyframe.percentage {
                        i = idx;
                        break;
                    }
                }
                if i < 1 {
                    continue;
                }
                if let Some(kf) = anim.keyframes.get(i) {
                    if let Some(prev_kf) = anim.keyframes.get(i - 1) {
                        this.apply_keyframe_at(kf, progress - prev_kf.percentage);
                    }
                }
            }
        }

        for name in to_remove {
            this.current_animations.remove(name);
        }
    }

    fn apply_keyframe(&mut self, keyframe: &Keyframe) {
        for (target, changes) in &keyframe.changes {
            if let Some(target) = self.part_lookup.as_ref().unwrap().get(*target) {
                let mut part = target.lock();
                let trans = &mut part.transform;
                for (path, value) in changes.iter() {
                    match path {
                        Path::Translate => trans.translation = value.as_vec2(),
                        Path::TranslateX => trans.translation.x = value.as_f32(),
                        Path::TranslateY => trans.translation.y = value.as_f32(),
                        Path::Scale => trans.scale = value.as_vec2(),
                        Path::ScaleX => trans.scale.x = value.as_f32(),
                        Path::ScaleY => trans.scale.y = value.as_f32(),
                        Path::Rotate => trans.rotation = value.as_f32().to_radians(),
                        Path::Origin => trans.origin = value.as_vec2(),
                        Path::OriginX => trans.origin.x = value.as_f32(),
                        Path::OriginY => trans.origin.y = value.as_f32(),
                    }
                }
                part.prev_transform = trans.clone();
            }
        }
    }

    fn apply_keyframe_at(&mut self, keyframe: &Keyframe, progress: f32) {
        for (target, changes) in &keyframe.changes {
            if let Some(target) = self.part_lookup.as_ref().unwrap().get(*target) {
                let mut part = target.lock();
                let mut trans = part.prev_transform.clone();
                for (path, value) in changes.iter() {
                    match path {
                        Path::Translate => trans.translation = value.as_vec2(),
                        Path::TranslateX => trans.translation.x = value.as_f32(),
                        Path::TranslateY => trans.translation.y = value.as_f32(),
                        Path::Scale => trans.scale = value.as_vec2(),
                        Path::ScaleX => trans.scale.x = value.as_f32(),
                        Path::ScaleY => trans.scale.y = value.as_f32(),
                        Path::Rotate => trans.rotation = value.as_f32().to_radians(),
                        Path::Origin => trans.origin = value.as_vec2(),
                        Path::OriginX => trans.origin.x = value.as_f32(),
                        Path::OriginY => trans.origin.y = value.as_f32(),
                    }
                }
                let prev = part.prev_transform.clone();
                let part_trans = &mut part.transform;
                part_trans.translation = Self::interpolate_vec2(prev.translation, trans.translation, progress);
                part_trans.scale = Self::interpolate_vec2(prev.scale, trans.scale, progress);
                part_trans.origin = Self::interpolate_vec2(prev.origin, trans.origin, progress);
                part_trans.rotation = Self::interpolate_f32(prev.rotation, trans.rotation, progress);
            }
        }
    }

    fn interpolate_f32(start: f32, end: f32, percent: f32) -> f32 {
        start + (end - start) * percent
    }

    fn interpolate_vec2(start: Vec2, end: Vec2, percent: f32) -> Vec2 {
        Vec2::new(start.x + (end.x - start.x) * percent, start.y + (end.y - start.y) * percent)
    }
}
