use crate::ui::geometry::shape::msfx::ty::{MappedVariable, Vec2};
use crate::ui::geometry::shape::{Shape, shapes};
use hashbrown::HashMap;
use mvengine_proc_macro::msfx_fn;

pub trait MSFXFunction {
    fn call_ordered(
        &self,
        arguments: HashMap<String, MappedVariable>,
        order: &[String],
    ) -> Result<MappedVariable, String> {
        self.call(arguments)
    }
    fn call(&self, arguments: HashMap<String, MappedVariable>) -> Result<MappedVariable, String>;
}

fn get_named(arguments: &HashMap<String, MappedVariable>, name: &str) -> MappedVariable {
    arguments.get(name).cloned().unwrap_or(MappedVariable::Null)
}

fn get_unnamed(arguments: &HashMap<String, MappedVariable>, name: &str) -> MappedVariable {
    let mut value = arguments.get(name);
    if value.is_none() {
        value = arguments.get("_");
    }
    value.cloned().unwrap_or(MappedVariable::Null)
}

struct Print;

impl MSFXFunction for Print {
    fn call_ordered(
        &self,
        arguments: HashMap<String, MappedVariable>,
        order: &[String],
    ) -> Result<MappedVariable, String> {
        let mut buf = String::new();
        for name in order {
            let value = arguments
                .get(name)
                .expect("This shouldn't happen... (MSFXParser must have a critical bug)");
            if name.starts_with("_") {
                buf.push_str(&format!("{}, ", value));
            } else {
                buf.push_str(&format!("{}: {}, ", name, value));
            }
        }
        buf.pop();
        buf.pop();
        println!("{buf}");
        Ok(MappedVariable::Null)
    }

    fn call(&self, _: HashMap<String, MappedVariable>) -> Result<MappedVariable, String> {
        unreachable!()
    }
}

struct Assert;

impl MSFXFunction for Assert {
    fn call(&self, arguments: HashMap<String, MappedVariable>) -> Result<MappedVariable, String> {
        let value = get_unnamed(&arguments, "value").as_bool()?;
        if value {
            Ok(MappedVariable::Null)
        } else {
            Err("Assertion failed!".to_string())
        }
    }
}

struct SameType;

impl MSFXFunction for SameType {
    fn call(&self, arguments: HashMap<String, MappedVariable>) -> Result<MappedVariable, String> {
        let a = get_named(&arguments, "a");
        let b = get_named(&arguments, "b");
        Ok(MappedVariable::Bool(a.ty() == b.ty()))
    }
}

struct Abs;

impl MSFXFunction for Abs {
    fn call(&self, arguments: HashMap<String, MappedVariable>) -> Result<MappedVariable, String> {
        let value = get_unnamed(&arguments, "value");
        match value {
            MappedVariable::Number(n) => Ok(MappedVariable::Number(n.abs())),
            MappedVariable::Vec2(v) => Ok(MappedVariable::Number(v.abs_sqr().sqrt())),
            MappedVariable::Bool(_) => Err("Invalid argument: Expected number but found bool!".to_string()),
            MappedVariable::Shape(_) => Err("Invalid argument: Expected number but found shape!".to_string()),
            MappedVariable::Null => Err("Invalid argument: Expected number but found null!".to_string()),
        }
    }
}

#[msfx_fn]
fn sin(value: f64) -> f64 {
    value.sin()
}

#[msfx_fn]
fn cos(value: f64) -> f64 {
    value.cos()
}

#[msfx_fn]
fn tan(value: f64) -> f64 {
    value.tan()
}

#[msfx_fn]
fn asin(value: f64) -> f64 {
    value.asin()
}

#[msfx_fn]
fn acos(value: f64) -> f64 {
    value.cos()
}

#[msfx_fn]
fn atan(value: f64) -> f64 {
    value.atan()
}

#[msfx_fn]
fn atan2(a: f64, b: f64) -> f64 {
    a.atan2(b)
}

#[msfx_fn]
fn floor(value: f64) -> f64 {
    value.floor()
}

#[msfx_fn]
fn ceil(value: f64) -> f64 {
    value.ceil()
}

#[msfx_fn]
fn clamp(value: f64, min: f64, max: f64) -> f64 {
    value.clamp(min, max)
}

#[msfx_fn]
fn sqrt(value: f64) -> f64 {
    value.sqrt()
}

#[msfx_fn]
fn exp(value: f64) -> f64 {
    value.exp()
}

#[msfx_fn]
fn exp2(value: f64) -> f64 {
    value.exp2()
}

#[msfx_fn]
fn ln(value: f64) -> f64 {
    value.ln()
}

#[msfx_fn]
fn log2(value: f64) -> f64 {
    value.log2()
}

#[msfx_fn]
fn log10(value: f64) -> f64 {
    value.log10()
}

#[msfx_fn]
fn round(value: f64) -> f64 {
    value.round()
}

#[msfx_fn]
fn trunc(value: f64) -> f64 {
    value.trunc()
}

#[msfx_fn]
fn fract(value: f64) -> f64 {
    value.fract()
}

#[msfx_fn]
fn sign(value: f64) -> f64 {
    value.signum()
}

#[msfx_fn]
fn min(a: f64, b: f64) -> f64 {
    a.min(b)
}

#[msfx_fn]
fn max(a: f64, b: f64) -> f64 {
    a.max(b)
}

#[msfx_fn]
fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

#[msfx_fn]
fn deg_to_rad(deg: f64) -> f64 {
    deg.to_radians()
}

#[msfx_fn]
fn rad_to_deg(rad: f64) -> f64 {
    rad.to_degrees()
}

#[msfx_fn]
fn is_nan(value: f64) -> bool {
    value.is_nan()
}

#[msfx_fn]
fn is_finite(value: f64) -> bool {
    value.is_finite()
}

#[msfx_fn]
fn is_infinite(value: f64) -> bool {
    value.is_infinite()
}

#[msfx_fn]
fn cbrt(value: f64) -> f64 {
    value.cbrt()
}

#[msfx_fn]
fn hypot(x: f64, y: f64) -> f64 {
    x.hypot(y)
}

#[msfx_fn]
fn recip(value: f64) -> f64 {
    value.recip()
}

#[msfx_fn]
fn copysign(magnitude: f64, sign: f64) -> f64 {
    magnitude.copysign(sign)
}

#[msfx_fn]
fn fma(a: f64, b: f64, c: f64) -> f64 {
    a.mul_add(b, c)
}

#[msfx_fn]
fn is_sign_positive(value: f64) -> bool {
    value.is_sign_positive()
}

#[msfx_fn]
fn is_sign_negative(value: f64) -> bool {
    value.is_sign_negative()
}

#[msfx_fn]
fn next_after(start: f64, direction: Option<f64>) -> f64 {
    if let Some(direction) = direction {
        if direction >= 0.0 {
            start.next_up()
        } else {
            start.next_down()
        }
    } else {
        start.next_up()
    }
}

#[msfx_fn]
fn rect0(x: f64, y: f64, width: f64, height: f64) -> Shape {
    shapes::rectangle0(x as i32, y as i32, width as i32, height as i32)
}

#[msfx_fn]
fn rect1(x1: f64, y1: f64, x2: f64, y2: f64) -> Shape {
    shapes::rectangle1(x1 as i32, y1 as i32, x2 as i32, y2 as i32)
}

/*#[msfx_fn]
fn rect2(rect: SimpleRect) -> Shape {
    shapes::rectangle2(rect)
}*/

#[msfx_fn]
fn arc0(
    cx: f64,
    cy: f64,
    radius: f64,
    offset_rad: Option<f64>,
    range_rad: Option<f64>,
    offset_deg: Option<f64>,
    range_deg: Option<f64>,
    tri_count: f64,
) -> Result<Shape, String> {
    let offset = if let Some(offset) = offset_rad {
        offset
    } else if let Some(offset) = offset_deg {
        offset.to_radians()
    } else {
        return Err(
            "arc0: You must specify either 'offset_rad' or 'offset_deg' parameter".to_string(),
        );
    };

    let range = if let Some(range) = range_rad {
        range
    } else if let Some(range) = range_deg {
        range.to_radians()
    } else {
        return Err(
            "arc0: You must specify either 'range_rad' or 'range_deg' parameter".to_string(),
        );
    };

    Ok(shapes::arc0(
        cx as i32,
        cy as i32,
        radius as i32,
        offset as f32,
        range as f32,
        tri_count as i32,
    ))
}

#[msfx_fn]
fn arc1(
    cx: f64,
    cy: f64,
    radius_x: f64,
    radius_y: f64,
    offset_rad: Option<f64>,
    range_rad: Option<f64>,
    offset_deg: Option<f64>,
    range_deg: Option<f64>,
    tri_count: f64,
) -> Result<Shape, String> {
    let offset = if let Some(offset) = offset_rad {
        offset
    } else if let Some(offset) = offset_deg {
        offset.to_radians()
    } else {
        return Err(
            "arc1: You must specify either 'offset_rad' or 'offset_deg' parameter".to_string(),
        );
    };
    let range = if let Some(range) = range_rad {
        range
    } else if let Some(range) = range_deg {
        range.to_radians()
    } else {
        return Err(
            "arc1: You must specify either 'range_rad' or 'range_deg' parameter".to_string(),
        );
    };
    Ok(shapes::arc1(
        cx as i32,
        cy as i32,
        radius_x as i32,
        radius_y as i32,
        offset as f32,
        range as f32,
        tri_count as i32,
    ))
}

#[msfx_fn]
fn circle0(cx: f64, cy: f64, radius: f64, tri_count: f64) -> Shape {
    shapes::circle0(cx as i32, cy as i32, radius as i32, tri_count as i32)
}

#[msfx_fn]
fn ellipse0(cx: f64, cy: f64, radius_x: f64, radius_y: f64, tri_count: f64) -> Shape {
    shapes::ellipse0(
        cx as i32,
        cy as i32,
        radius_x as i32,
        radius_y as i32,
        tri_count as i32,
    )
}

#[msfx_fn]
fn triangle0(x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64) -> Shape {
    shapes::triangle0(
        x1 as i32, y1 as i32, x2 as i32, y2 as i32, x3 as i32, y3 as i32,
    )
}

#[msfx_fn]
fn triangle2(v1: Vec2, v2: Vec2, v3: Vec2) -> Shape {
    shapes::triangle2(v1.as_mvengine(), v2.as_mvengine(), v3.as_mvengine())
}

#[msfx_fn(Vec2Fn)]
fn vec2(__actual_literal_underscore_lmao: Option<f64>, value: Option<f64>, x: Option<f64>, y: Option<f64>) -> Result<Vec2, String> {
    if let Some(x) = x && let Some(y) = y {
        Ok(Vec2::new(x, y))
    } else if let Some(value) = __actual_literal_underscore_lmao {
        Ok(Vec2::splat(value))
    } else if let Some(value) = value {
        Ok(Vec2::splat(value))
    } else {
        Err("vec2 function requires either unnamed variable or BOTH 'x' and 'y' variables".to_string())
    }
}

#[msfx_fn]
fn vec2_len(v: Vec2) -> f64 {
    v.abs_sqr().sqrt()
}

#[msfx_fn]
fn vec2_len_sq(v: Vec2) -> f64 {
    v.abs_sqr()
}

#[msfx_fn]
fn vec2_normalize(v: Vec2) -> Vec2 {
    let len = v.abs_sqr().sqrt();
    if len == 0.0 {
        Vec2 { x: 0.0, y: 0.0 }
    } else {
        Vec2 {
            x: v.x / len,
            y: v.y / len,
        }
    }
}

#[msfx_fn]
fn vec2_dot(a: Vec2, b: Vec2) -> f64 {
    a.x * b.x + a.y * b.y
}

#[msfx_fn]
fn vec2_perp(v: Vec2) -> Vec2 {
    Vec2 {
        x: -v.y,
        y: v.x,
    }
}

#[msfx_fn]
fn vec2_lerp(a: Vec2, b: Vec2, t: f64) -> Vec2 {
    Vec2 {
        x: a.x + (b.x - a.x) * t,
        y: a.y + (b.y - a.y) * t,
    }
}

#[msfx_fn]
fn vec2_clamp(v: Vec2, min: Vec2, max: Vec2) -> Vec2 {
    Vec2 {
        x: v.x.clamp(min.x, max.x),
        y: v.y.clamp(min.y, max.y),
    }
}

#[msfx_fn]
fn vec2_angle(v: Vec2) -> f64 {
    v.y.atan2(v.x)
}

#[msfx_fn]
fn vec2_rotate(v: Vec2, angle_rad: f64) -> Vec2 {
    let cos_theta = angle_rad.cos();
    let sin_theta = angle_rad.sin();
    Vec2 {
        x: v.x * cos_theta - v.y * sin_theta,
        y: v.x * sin_theta + v.y * cos_theta,
    }
}

#[msfx_fn]
fn vec2_reflect(v: Vec2, normal: Vec2) -> Vec2 {
    let dot = v.x * normal.x + v.y * normal.y;
    Vec2 {
        x: v.x - 2.0 * dot * normal.x,
        y: v.y - 2.0 * dot * normal.y,
    }
}

#[msfx_fn]
fn vec2_project(v: Vec2, onto: Vec2) -> Vec2 {
    let onto_len_squared = onto.abs_sqr();
    if onto_len_squared == 0.0 {
        Vec2 { x: 0.0, y: 0.0 }
    } else {
        let dot = v.x * onto.x + v.y * onto.y;
        let scale = dot / onto_len_squared;
        Vec2 {
            x: onto.x * scale,
            y: onto.y * scale,
        }
    }
}

pub fn get_function(name: &str) -> Option<Box<dyn MSFXFunction>> {
    match name {
        "print" => Some(Box::new(Print)),
        "assert" => Some(Box::new(Assert)),
        "sameType" => Some(Box::new(SameType)),
        "same_type" => Some(Box::new(SameType)),
        "sin" => Some(Box::new(Sin)),
        "cos" => Some(Box::new(Cos)),
        "tan" => Some(Box::new(Tan)),
        "asin" => Some(Box::new(Asin)),
        "acos" => Some(Box::new(Acos)),
        "atan" => Some(Box::new(Atan)),
        "atan2" => Some(Box::new(Atan2)),
        "floor" => Some(Box::new(Floor)),
        "ceil" => Some(Box::new(Ceil)),
        "clamp" => Some(Box::new(Clamp)),
        "abs" => Some(Box::new(Abs)),
        "sqrt" => Some(Box::new(Sqrt)),
        "cbrt" => Some(Box::new(Cbrt)),
        "hypot" => Some(Box::new(Hypot)),
        "recip" => Some(Box::new(Recip)),
        "copysign" => Some(Box::new(Copysign)),
        "fma" => Some(Box::new(Fma)),
        "ln" => Some(Box::new(Ln)),
        "log10" => Some(Box::new(Log10)),
        "log2" => Some(Box::new(Log2)),
        "exp" => Some(Box::new(Exp)),
        "exp2" => Some(Box::new(Exp2)),
        "round" => Some(Box::new(Round)),
        "trunc" => Some(Box::new(Trunc)),
        "fract" => Some(Box::new(Fract)),
        "sign" => Some(Box::new(Sign)),
        "is_sign_positive" => Some(Box::new(IsSignPositive)),
        "is_sign_negative" => Some(Box::new(IsSignNegative)),
        "next_after" => Some(Box::new(NextAfter)),
        "min" => Some(Box::new(Min)),
        "max" => Some(Box::new(Max)),
        "lerp" => Some(Box::new(Lerp)),
        "deg_to_rad" => Some(Box::new(DegToRad)),
        "rad_to_deg" => Some(Box::new(RadToDeg)),
        "is_nan" => Some(Box::new(IsNan)),
        "is_finite" => Some(Box::new(IsFinite)),
        "is_infinite" => Some(Box::new(IsInfinite)),
        "rect0" => Some(Box::new(Rect0)),
        "rect1" => Some(Box::new(Rect1)),
        // "rect2" => Some(Box::new(Rect2)),
        "arc0" => Some(Box::new(Arc0)),
        "arc1" => Some(Box::new(Arc1)),
        "circle0" => Some(Box::new(Circle0)),
        "ellipse0" => Some(Box::new(Ellipse0)),
        "triangle0" => Some(Box::new(Triangle0)),
        "triangle2" => Some(Box::new(Triangle2)),
        "vec2" => Some(Box::new(Vec2Fn)),
        "vec2_len" => Some(Box::new(Vec2Len)),
        "vec2_len_sq" => Some(Box::new(Vec2LenSq)),
        "vec2_normalize" => Some(Box::new(Vec2Normalize)),
        "vec2_dot" => Some(Box::new(Vec2Dot)),
        "vec2_perp" => Some(Box::new(Vec2Perp)),
        "vec2_lerp" => Some(Box::new(Vec2Lerp)),
        "vec2_clamp" => Some(Box::new(Vec2Clamp)),
        "vec2_angle" => Some(Box::new(Vec2Angle)),
        "vec2_rotate" => Some(Box::new(Vec2Rotate)),
        "vec2_reflect" => Some(Box::new(Vec2Reflect)),
        "vec2_project" => Some(Box::new(Vec2Project)),
        _ => None,
    }
}
