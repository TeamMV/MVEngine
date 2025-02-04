use std::thread::current;
use hashbrown::HashMap;
use mvutils::unsafe_utils::Unsafe;
use crate::math::vec::Vec2;
use crate::rendering::Transform;
use crate::ui::rendering::adaptive::AdaptiveShape;
use crate::ui::rendering::ctx;
use crate::ui::rendering::ctx::DrawShape;
use crate::ui::rendering::shapes::{Assignment, Ast, Command, Param, ParsedStruct, StructValue};
use crate::ui::rendering::shapes::modifier::boolean;

fn parse_vec2(parsed_struct: &ParsedStruct) -> Result<Vec2, String> {
    let mut vec2 = Vec2::new(0.0, 0.0);
    for (key, value) in &parsed_struct.values {
        let skey = key.as_str();
        match skey {
            "x" => {
                if let StructValue::Number(num) = value {
                    vec2.x = num.as_f32();
                } else {
                    return Err("x can only be a number, found struct".to_string());
                }
            }
            "y" => {
                if let StructValue::Number(num) = value {
                    vec2.y = num.as_f32();
                } else {
                    return Err("y can only be a number, found struct".to_string());
                }
            }
            _ => return Err(format!("Unrecognized parameter {skey}"))
        }
    }
    Ok(vec2)
}

fn parse_transform(parsed_struct: &ParsedStruct) -> Result<Transform, String> {
    let mut transform = Transform::new();
    for (key, value) in &parsed_struct.values {
        let skey = key.as_str();
        match skey {
            "r" => {
                if let StructValue::Number(num) = value {
                    transform.rotation = num.as_f32();
                } else {
                    return Err("rotation can only be a number, found struct".to_string());
                }
            }
            "t" => {
                if let StructValue::Struct(parsed_struct) = value {
                    transform.translation = parse_vec2(parsed_struct.as_ref())?;
                } else {
                    return Err("translation can only be a struct, found number".to_string());
                }
            }
            "s" => {
                if let StructValue::Struct(parsed_struct) = value {
                    transform.scale = parse_vec2(parsed_struct.as_ref())?;
                } else {
                    return Err("scale can only be a struct, found number".to_string());
                }
            }
            "o" => {
                if let StructValue::Struct(parsed_struct) = value {
                    transform.origin = parse_vec2(parsed_struct.as_ref())?;
                } else {
                    return Err("origin can only be a struct, found number".to_string());
                }
            }
            _ => return Err(format!("Unrecognized parameter {skey}"))
        }
    }
    Ok(transform)
}

fn gen_tri(parsed_struct: ParsedStruct) -> Result<DrawShape, String> {
    let mut p1 = (0, 0);
    let mut p2 = (0, 0);
    let mut p3 = (0, 0);
    let mut transform = Transform::new();

    for (key, value) in &parsed_struct.values {
        let skey = key.as_str();
        match skey {
            "a" => {
                if let StructValue::Struct(parsed_struct) = value {
                    p1 = parse_vec2(parsed_struct.as_ref())?.as_i32_tuple();
                } else {
                    return Err("p1 can only be a struct, found number".to_string());
                }
            }
            "b" => {
                if let StructValue::Struct(parsed_struct) = value {
                    p2 = parse_vec2(parsed_struct.as_ref())?.as_i32_tuple();
                } else {
                    return Err("p2 can only be a struct, found number".to_string());
                }
            }
            "c" => {
                if let StructValue::Struct(parsed_struct) = value {
                    p3 = parse_vec2(parsed_struct.as_ref())?.as_i32_tuple();
                } else {
                    return Err("p3 can only be a struct, found number".to_string());
                }
            }
            "t" => {
                if let StructValue::Struct(parsed_struct) = value {
                    transform = parse_transform(parsed_struct.as_ref())?;
                } else {
                    return Err("p3 can only be a struct, found number".to_string());
                }
            }
            _ => return Err(format!("Unrecognized parameter {skey}"))
        }
    }
    let mut triangle = ctx::triangle()
        .point(p1, None)
        .point(p2, None)
        .point(p3, None)
        .create();
    triangle.set_transform(transform);
    Ok(triangle)
}

fn gen_rect(parsed_struct: ParsedStruct) -> Result<DrawShape, String> {
    let mut p1 = (0, 0);
    let mut p2 = (0, 0);
    let mut wh = (0, 0);
    let mut is_wh = false;
    let mut transform = Transform::new();

    for (key, value) in &parsed_struct.values {
        let skey = key.as_str();
        match skey {
            "x" => {
                if let StructValue::Number(num) = value {
                    p1.0 = num.as_i32();
                } else {
                    return Err("rotation can only be a number, found struct".to_string());
                }
            }
            "y" => {
                if let StructValue::Number(num) = value {
                    p1.1 = num.as_i32();
                } else {
                    return Err("rotation can only be a number, found struct".to_string());
                }
            }
            "w" => {
                if let StructValue::Number(num) = value {
                    wh.0 = num.as_i32();
                    is_wh = true;
                } else {
                    return Err("rotation can only be a number, found struct".to_string());
                }
            }
            "h" => {
                if let StructValue::Number(num) = value {
                    wh.1 = num.as_i32();
                    is_wh = true;
                } else {
                    return Err("rotation can only be a number, found struct".to_string());
                }
            }
            "a" => {
                if let StructValue::Struct(parsed_struct) = value {
                    p1 = parse_vec2(parsed_struct.as_ref())?.as_i32_tuple();
                } else {
                    return Err("p1 can only be a struct, found number".to_string());
                }
            }
            "b" => {
                if let StructValue::Struct(parsed_struct) = value {
                    p2 = parse_vec2(parsed_struct.as_ref())?.as_i32_tuple();
                } else {
                    return Err("p2 can only be a struct, found number".to_string());
                }
            }
            "t" => {
                if let StructValue::Struct(parsed_struct) = value {
                    transform = parse_transform(parsed_struct.as_ref())?;
                } else {
                    return Err("p3 can only be a struct, found number".to_string());
                }
            }
            _ => return Err(format!("Unrecognized parameter {skey}"))
        }
    }

    if is_wh {
        p2.0 = p1.0 + wh.0;
        p2.1 = p1.1 + wh.1;
    }

    let mut rectangle = ctx::rectangle()
        .xyxy(p1.0, p1.1, p2.0, p2.1)
        .create();
    rectangle.set_transform(transform);
    Ok(rectangle)
}

fn gen_arc(parsed_struct: ParsedStruct) -> Result<DrawShape, String> {
    let mut angle = 0.0;
    let mut radius = 0;
    let mut triangle_count = 50;
    let mut center = (0, 0);
    let mut transform = Transform::new();

    for (key, value) in &parsed_struct.values {
        let skey = key.as_str();

        match skey {
            "a" => {
                if let StructValue::Number(num) = value {
                    angle = num.as_f32();
                } else {
                    return Err("rotation can only be a number, found struct".to_string());
                }
            }
            "r" => {
                if let StructValue::Number(num) = value {
                    radius = num.as_i32();
                } else {
                    return Err("rotation can only be a number, found struct".to_string());
                }
            }
            "tc" => {
                if let StructValue::Number(num) = value {
                    triangle_count = num.as_i32();
                } else {
                    return Err("rotation can only be a number, found struct".to_string());
                }
            }
            "c" => {
                if let StructValue::Struct(parsed_struct) = value {
                    center = parse_vec2(parsed_struct.as_ref())?.as_i32_tuple();
                } else {
                    return Err("p1 can only be a struct, found number".to_string());
                }
            }
            "t" => {
                if let StructValue::Struct(parsed_struct) = value {
                    transform = parse_transform(parsed_struct.as_ref())?;
                } else {
                    return Err("p3 can only be a struct, found number".to_string());
                }
            }
            _ => return Err(format!("Unrecognized parameter {skey}"))
        }
    }

    let mut arc = ctx::arc()
        .center(center.0, center.1)
        .angle(angle)
        .radius(radius)
        .triangle_count(triangle_count as u32)
        .create();
    arc.set_transform(transform);
    Ok(arc)
}

pub struct ShapeGenerator;

impl ShapeGenerator {
    pub fn generate(ast: Ast) -> Result<DrawShape, String> {
        let mut vars: HashMap<String, DrawShape> = HashMap::new();
        let vars2 = unsafe { Unsafe::cast_static(&vars) };
        let mut current_selection = String::new();

        for command in ast {
            match command {
                Command::Assign(name, assignment) => {
                    match assignment {
                        Assignment::New(parsed_struct) => {
                            let prim_name = &parsed_struct.name;
                            let shape = match prim_name.as_str() {
                                "tri" => gen_tri(parsed_struct)?,
                                "rect" => gen_rect(parsed_struct)?,
                                "arc" => gen_arc(parsed_struct)?,
                                _ => return Err(format!("Unknown primitve {prim_name}. Did you mean [tri, rect, arc]?").to_string())
                            };
                            vars.insert(name, shape);
                        }
                        Assignment::Clone(other) => {
                            if let Some(shape) = vars.get(&other).cloned() {
                                vars.insert(name, shape);
                            } else {
                                return Err(format!("{other} is not defined!"));
                            }
                        }
                    }
                }
                Command::Select(name) => {
                    if let Some(_) = vars.get_mut(&name) {
                        current_selection = name;
                    } else {
                        return Err(format!("{name} is not defined!"));
                    }
                }
                Command::Call(function, params) => {
                    match function.as_ref() {
                        "transform" => {
                            let current = vars.get_mut(&current_selection).ok_or("No shape selected".to_string())?;
                            let first = params.get(0).ok_or("tranform needs one transformation struct".to_string())?;
                            if let Param::Struct(parsed_struct) = first {
                                let transform = parse_transform(parsed_struct)?;
                                current.set_transform(transform);
                            }
                        }
                        "apply" => {
                            let current = vars.get_mut(&current_selection).ok_or("No shape selected".to_string())?;
                            current.apply_transformations();
                        }
                        "recenter" => {
                            let current = vars.get_mut(&current_selection).ok_or("No shape selected".to_string())?;
                            current.recenter();
                        }
                        "combine" => {
                            let first = params.get(0).ok_or("combine needs one other shape".to_string())?;
                            if let Param::Str(other) = first {
                                let other_shape = vars.get(other).ok_or(format!("{other} is not defined!"))?;
                                let other_shape = unsafe { Unsafe::cast_static(other_shape) };
                                let current = vars.get_mut(&current_selection).ok_or("No shape selected".to_string())?;
                                current.combine(other_shape);
                            }
                        }
                        "export" => {
                            let all_param = Param::Str("all".to_string());
                            let first = params.get(0).unwrap_or(&all_param);
                            if let Param::Str(export) = first {

                            }
                            let current = vars.get_mut(&current_selection).ok_or("No shape selected".to_string())?;
                            return Ok(current.clone());
                        }
                        "modifier" => {
                            let modifier = params.get(0).ok_or("modifier needs one modifier name argument".to_string())?;
                            if let Param::Str(name) = modifier {
                                let mod_params = params.get(1..).unwrap_or_default().to_vec();
                                let current = vars.get_mut(&current_selection).ok_or("No shape selected".to_string())?;
                                match name.as_str() {
                                    "Boolean" => unsafe {
                                        let res = boolean::compute(current, mod_params, vars2)?;
                                        vars.insert(current_selection.clone(), res);
                                    }
                                    _ => return Err(format!("Modifier {name} is not defined!"))
                                }
                            }
                        }
                        _ => return Err(format!("Function {function} is not defined!"))
                    }
                }
            }
        }

        Err("No shape exported".to_string())
    }

    pub fn generate_adaptive(ast: Ast) -> Result<AdaptiveShape, String> {
        let mut vars: HashMap<String, DrawShape> = HashMap::new();
        let vars2 = unsafe { Unsafe::cast_static(&vars) };
        let mut current_selection = String::new();

        let mut parts = [0; 9].map(|_| None);

        for command in ast {
            match command {
                Command::Assign(name, assignment) => {
                    match assignment {
                        Assignment::New(parsed_struct) => {
                            let prim_name = &parsed_struct.name;
                            let shape = match prim_name.as_str() {
                                "tri" => gen_tri(parsed_struct)?,
                                "rect" => gen_rect(parsed_struct)?,
                                "arc" => gen_arc(parsed_struct)?,
                                _ => return Err(format!("Unknown primitve {prim_name}. Did you mean [tri, rect, arc]?").to_string())
                            };
                            vars.insert(name, shape);
                        }
                        Assignment::Clone(other) => {
                            if let Some(shape) = vars.get(&other).cloned() {
                                vars.insert(name, shape);
                            } else {
                                return Err(format!("{other} is not defined!"));
                            }
                        }
                    }
                }
                Command::Select(name) => {
                    if let Some(_) = vars.get_mut(&name) {
                        current_selection = name;
                    } else {
                        return Err(format!("{name} is not defined!"));
                    }
                }
                Command::Call(function, params) => {
                    match function.as_ref() {
                        "transform" => {
                            let current = vars.get_mut(&current_selection).ok_or("No shape selected".to_string())?;
                            let first = params.get(0).ok_or("tranform needs one transformation struct".to_string())?;
                            if let Param::Struct(parsed_struct) = first {
                                let transform = parse_transform(parsed_struct)?;
                                current.set_transform(transform);
                            }
                        }
                        "apply" => {
                            let current = vars.get_mut(&current_selection).ok_or("No shape selected".to_string())?;
                            current.apply_transformations();
                        }
                        "recenter" => {
                            let current = vars.get_mut(&current_selection).ok_or("No shape selected".to_string())?;
                            current.recenter();
                        }
                        "combine" => {
                            let first = params.get(0).ok_or("combine needs one other shape".to_string())?;
                            if let Param::Str(other) = first {
                                let other_shape = vars.get(other).ok_or(format!("{other} is not defined!"))?;
                                let other_shape = unsafe { Unsafe::cast_static(other_shape) };
                                let current = vars.get_mut(&current_selection).ok_or("No shape selected".to_string())?;
                                current.combine(other_shape);
                            }
                        }
                        "export" => {
                            let all_param = Param::Str("finish".to_string());
                            let first = params.get(0).unwrap_or(&all_param);
                            if let Param::Str(export) = first {
                                let current = vars.get(&current_selection).ok_or("No shape selected".to_string())?;
                                match export.as_str() {
                                    "finish" => return Ok(AdaptiveShape::from_arr(parts)),
                                    "bl" | "bottom_left" => parts[0] = Some(current.clone()),
                                    "tl" | "top_left" => parts[2] = Some(current.clone()),
                                    "tr" | "top_right" => parts[4] = Some(current.clone()),
                                    "br" | "bottom_right" => parts[6] = Some(current.clone()),
                                    "l" | "left" => parts[1] = Some(current.clone()),
                                    "t" | "top" => parts[3] = Some(current.clone()),
                                    "r" | "right" => parts[5] = Some(current.clone()),
                                    "b" | "bottom" => parts[7] = Some(current.clone()),
                                    "c" | "center" => parts[8] = Some(current.clone()),
                                    _ => return Err(format!("Invalid export: {export}"))
                                }
                            }
                        }
                        "modifier" => {
                            let modifier = params.get(0).ok_or("modifier needs one modifier name argument".to_string())?;
                            if let Param::Str(name) = modifier {
                                let mod_params = params.get(1..).unwrap_or_default().to_vec();
                                let current = vars.get_mut(&current_selection).ok_or("No shape selected".to_string())?;
                                match name.as_str() {
                                    "Boolean" => unsafe {
                                        let res = boolean::compute(current, mod_params, vars2)?;
                                        vars.insert(current_selection.clone(), res);
                                    }
                                    _ => return Err(format!("Modifier {name} is not defined!"))
                                }
                            }
                        }
                        _ => return Err(format!("Function {function} is not defined!"))
                    }
                }
            }
        }

        Err("No shape exported".to_string())
    }
}