use crate::color::RgbColor;
use crate::graphics::comp::parse::rig::{BoneStart, Parsed, ParsedBone, ParsedPart, ParsedRig};
use crate::math::vec::Vec2;
use crate::rendering::RenderContext;
use crate::ui::context::UiResources;
use crate::ui::geometry::{geom, Rect, SimpleRect};
use crate::ui::rendering::adaptive::AdaptiveFill;
use crate::ui::rendering::ctx;
use crate::ui::rendering::ctx::DrawContext2D;
use crate::ui::res::MVR;
use crate::utils::savers::SaveArc;
use crate::window::Window;
use hashbrown::HashMap;
use itertools::Itertools;
use mvutils::unsafe_utils::Unsafe;
use mvutils::Savable;
use parking_lot::RwLock;
use std::fmt::{Debug, Formatter};

/// Yeah the savable will be broken as hell because each arc will have its own instance behind it
#[derive(Clone, Savable)]
pub struct Rig {
    pub root_bone: BoneRc,
    pub skeleton: Skeleton
}

impl Rig {
    pub fn from_parsed(parsed_rig: ParsedRig) -> Result<Self, String> {
        let mut skeleton = Skeleton::new();

        let root = parsed_rig.bones.map
            .values()
            .filter(|b| if let BoneStart::Point(_) = b.start { true } else { false })
            .next();

        if let Some(root) = root {
            let bone = Bone::new(root, &mut skeleton, &parsed_rig.bones,&parsed_rig.parts)?;

            skeleton.compute_area();

            Ok(Self {
                root_bone: bone,
                skeleton,
            })
        } else {
            Err("No root bone found! Note: For a bone to be the root, it must start at a point rather than another bone!".to_string())
        }
    }

    pub fn debug_draw(&self, ctx: &mut DrawContext2D, area: &SimpleRect, window: &Window) {
        let skeleton_rect = &self.skeleton.area;
        for bone in self.skeleton.bones.values() {
            bone.write().debug_draw(ctx, area, skeleton_rect, window);
        }
    }
    
    pub fn draw(&self, ctx: &mut impl RenderContext, r: &'static (impl UiResources + ?Sized), area: &SimpleRect) {
        let mut l = self.root_bone.write();
        l.draw(ctx, r, area, &self.skeleton);
    }
}

#[derive(Clone, Savable)]
pub struct Bone {
    start: Vec2,
    end: Vec2,
    rotation: f32,
    length: f32,
    children: Vec<BoneRc>,
    aim_target: Option<Vec2>,
    parts: Vec<PartRc>
}

impl Bone {
    pub fn new(parsed: &ParsedBone, skeleton: &mut Skeleton, bones: &Parsed<ParsedBone>, parts: &Parsed<ParsedPart>) -> Result<BoneRc, String> {
        let start = match &parsed.start {
            BoneStart::Other(other) => {
                if let Some(b) = skeleton.bones.get(other) {
                    b.read().end
                } else {
                    return Err(format!("{other} bone does not exist!"));
                }
            }
            BoneStart::Point(pt) => {
                *pt
            }
        };

        let length = geom::distance(start, parsed.end);
        

        let this = Self {
            start,
            end: parsed.end,
            rotation: geom::angle_between_points(start, parsed.end),
            length,
            children: vec![],
            aim_target: None,
            parts: vec![],
        };

        let rc = SaveArc::new(RwLock::new(this));
        skeleton.bones.insert(parsed.name.clone(), rc.clone());

        let mut attached_parts = vec![];
        for attached in parts.map.values().filter(|p| p.bone == Some(parsed.name.clone())) {
            let name = attached.name.clone();
            let part = Part::new(attached, rc.clone());
            skeleton.parts.insert(name, part.clone());
            attached_parts.push(part);
        }

        let mut children = vec![];
        for child_bone in bones.map.values().filter(|b| {
            if let BoneStart::Other(other) = &b.start {
                other == parsed.name.as_str()
            } else {
                false
            }
        }) {
            //every bone that starts here
            let bone = Bone::new(child_bone, skeleton, bones, parts)?;
            children.push(bone);
        }

        rc.write().children = children;
        rc.write().parts = attached_parts;

        Ok(rc)
    }
    
    pub fn set_aim(&mut self, p: Vec2) {
        self.aim_target = Some(p);
    }
    
    pub fn clear_aim(&mut self) {
        self.aim_target = None;
    }
    
    pub fn set_rotation(&mut self, angle: f32) {
        self.rotate(angle - self.rotation);
    }

    pub fn rotate(&mut self, angle: f32) {
        self.rotation += angle;

        let dir = Vec2 { x: 0.0, y: self.length };

        let cos_theta = self.rotation.cos();
        let sin_theta = self.rotation.sin();

        let rotated = Vec2 {
            x: dir.x * cos_theta - dir.y * sin_theta,
            y: dir.x * sin_theta + dir.y * cos_theta,
        };

        self.end = geom::add(self.start, rotated);

        for child_bone in &self.children {
            child_bone.write().rotated_by_parent(self.end, angle);
        }

        let this = unsafe { Unsafe::cast_static(self) };
        for part in &mut self.parts {
            part.write().update(this);
        }
    }

    pub fn rotated_by_parent(&mut self, start: Vec2, rotation: f32) {
        self.start = start;
        self.rotation += rotation;

        let dir = Vec2 { x: 0.0, y: self.length };

        let cos_theta = self.rotation.cos();
        let sin_theta = self.rotation.sin();

        let rotated = Vec2 {
            x: dir.x * cos_theta - dir.y * sin_theta,
            y: dir.x * sin_theta + dir.y * cos_theta,
        };

        self.end = geom::add(self.start, rotated);

        for child in &self.children {
            child.write().rotated_by_parent(self.end, rotation);
        }

        let this = unsafe { Unsafe::cast_static(self) };
        for part in &mut self.parts {
            part.write().update(this);
        }
    }
    
    fn before_draw(&mut self, area: &SimpleRect, skeleton_area: &SimpleRect) {
        if let Some(aim_p) = self.aim_target {
            let start = geom::remap_point(self.start, skeleton_area, area);
            let rot = geom::angle_between_points(start, aim_p);
            self.set_rotation(rot);
        }
    }

    fn draw(&mut self, ctx: &mut impl RenderContext, r: &'static (impl UiResources + ?Sized), area: &SimpleRect, skeleton: &Skeleton) {
        self.before_draw(area, &skeleton.area);
        for part in &self.parts {
            let lock = part.read();
            if let Some(drawable_res) = lock.drawable {
                if let Some(drawable) = r.resolve_drawable(drawable_res) {
                    let mut rect = lock.create_rect();
                    rect.project(&skeleton.area, area);
                    drawable.draw(ctx, rect, r);
                }
            }
        }
        for child in &self.children {
            let mut l = child.write();
            l.draw(ctx, r, area, skeleton);
        }
    }

    pub fn debug_draw(&mut self, ctx: &mut DrawContext2D, area: &SimpleRect, skeleton_area: &SimpleRect, window: &Window) {
        self.before_draw(area, skeleton_area);
        
        let width = 6.0;

        let dir = geom::normalize(geom::sub(self.end, self.start));
        let perp = geom::perpendicular(dir);
        let base_left = geom::add(self.start, geom::scale(perp, width));
        let base_right = geom::sub(self.start, geom::scale(perp, width));
        let tip = self.end;

        let base_left = geom::remap_point(base_left, skeleton_area, area);
        let base_right = geom::remap_point(base_right, skeleton_area, area);
        let tip = geom::remap_point(tip, skeleton_area, area);

        let tri = ctx::triangle()
            .color(MVR.resolve_color(MVR.color.bone_debug).unwrap().clone())
            .point(base_left.as_i32_tuple(), None)
            .point(base_right.as_i32_tuple(), None)
            .point(tip.as_i32_tuple(), None)
            .create();

        ctx.shape(tri);

        for part in &self.parts {
            let mut rect = part.read().create_rect();
            rect.project(skeleton_area, area);
            if let Some(void_rect) = MVR.resolve_adaptive(MVR.adaptive.void_rect) {
                void_rect.draw(&mut *ctx, &rect, AdaptiveFill::Color(RgbColor::blue()), &window.ui().context(), &window.area());
            }
        }
    }
}

type BoneRc = SaveArc<RwLock<Bone>>;
type PartRc = SaveArc<RwLock<Part>>;

#[derive(Clone, Savable)]
pub struct Skeleton {
    pub bones: HashMap<String, BoneRc>,
    pub parts: HashMap<String, PartRc>,
    area: SimpleRect
}

impl Skeleton {
    pub fn new() -> Self {
        Self {
            bones: HashMap::new(),
            parts: HashMap::new(),
            area: SimpleRect::new(0, 0, 0, 0),
        }
    }

    fn compute_area(&mut self) {
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x: f32 = 0.0;
        let mut max_y: f32 = 0.0;

        for bone in self.bones.values() {
            let lock = bone.read();
            min_x = min_x.min(lock.start.x);
            min_x = min_x.min(lock.end.x);

            min_y = min_y.min(lock.start.y);
            min_y = min_y.min(lock.end.y);

            max_x = max_x.max(lock.start.x);
            max_x = max_x.max(lock.end.x);

            max_y = max_y.max(lock.start.y);
            max_y = max_y.max(lock.end.y);
        }

        let min_x = min_x as i32;
        let max_x = max_x as i32;
        let min_y = min_y as i32;
        let max_y = max_y as i32;

        self.area = SimpleRect::new(min_x, min_y, max_x - min_x, max_y - min_y);
    }
}

#[derive(Clone, Savable)]
pub struct Part {
    original_anchor: Vec2,
    position: Vec2,
    size: Vec2,
    rotation: f32,
    anchor: Vec2,
    drawable: Option<usize> //R.drawable
}

impl Part {
    pub fn new(parsed: &ParsedPart, bone: BoneRc) -> PartRc {
        let bind = bone.clone();
        let lock = bind.read();
        let mut this = Self {
            original_anchor: parsed.anchor,
            position: Vec2::default(),
            size: parsed.size,
            rotation: 0.0,
            anchor: parsed.anchor,
            drawable: None,
        };

        this.update(&*lock);
        
        SaveArc::new(RwLock::new(this))
    }

    fn update(&mut self, bone: &Bone) {
        self.rotation = bone.rotation;
        self.anchor = bone.start;

        self.position = geom::sub(self.anchor, self.original_anchor);
    }

    pub fn create_rect(&self) -> Rect {
        Rect::new(self.position.x as i32, self.position.y as i32, self.size.x as i32, self.size.y as i32, -self.rotation.to_degrees(), self.anchor.as_i32_tuple())
    }

    pub fn set_drawable(&mut self, drawable: Option<usize>) {
        self.drawable = drawable;
    }
}

impl Debug for Part {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Part")
            .field("position", &self.position)
            .field("size", &self.size)
            .field("rotation", &self.rotation)
            .field("anchor", &self.anchor)
            .finish()
    }
}