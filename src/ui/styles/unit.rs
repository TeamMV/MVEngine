use mvutils::Savable;

#[derive(Clone, Copy, Savable)]
pub enum Unit {
    Px(i32),             // px
    MM(f32),             // mm
    CM(f32),             // cm
    M(f32),              // m
    In(f32),             // in
    Twip(f32),           // twip
    Mil(f32),            // mil
    Point(f32),          // pt
    Pica(f32),           // pica
    Foot(f32),           // ft
    Yard(f32),           // yd
    Link(f32),           // lk
    Rod(f32),            // rd
    Chain(f32),          // ch
    Line(f32),           // ln
    BarleyCorn(f32),     // bc
    Nail(f32),           // nl
    Finger(f32),         // fg
    Stick(f32),          // sk
    Palm(f32),           // pm
    Shaftment(f32),      // sf
    Span(f32),           // sp
    Quarter(f32),        // qr
    Pace(f32),           // pc
    BeardFortnight(f32), //bf
}

impl Unit {
    pub fn as_px(&self, dpi: f32) -> i32 {
        match self {
            Unit::Px(px) => *px,
            Unit::MM(value) => ((value / 25.4) * dpi) as i32,
            Unit::CM(value) => ((value / 2.54) * dpi) as i32,
            Unit::M(value) => (value * dpi) as i32,
            Unit::In(value) => (value * dpi) as i32,
            Unit::Twip(value) => ((value / 1440.0) * dpi) as i32,
            Unit::Mil(value) => ((value / 1000.0) * dpi) as i32,
            Unit::Point(value) => (value * (dpi / 72.0)) as i32,
            Unit::Pica(value) => (value * (dpi / 6.0)) as i32,
            Unit::Foot(value) => ((value * 12.0) * dpi) as i32,
            Unit::Yard(value) => ((value * 36.0) * dpi) as i32,
            Unit::Link(value) => ((value * 7.92) * dpi) as i32,
            Unit::Rod(value) => ((value * 198.0) * dpi) as i32,
            Unit::Chain(value) => ((value * 792.0) * dpi) as i32,
            Unit::Line(value) => ((value * (1.0 / 40.0)) * dpi) as i32,
            Unit::BarleyCorn(value) => ((value * 0.125) * dpi) as i32,
            Unit::Nail(value) => ((value * 0.25) * dpi) as i32,
            Unit::Finger(value) => ((value * 0.375) * dpi) as i32,
            Unit::Stick(value) => ((value * 0.5) * dpi) as i32,
            Unit::Palm(value) => ((value * 3.0) * dpi) as i32,
            Unit::Shaftment(value) => ((value * 6.0) * dpi) as i32,
            Unit::Span(value) => ((value * 9.0) * dpi) as i32,
            Unit::Quarter(value) => ((value * 36.0) * dpi) as i32,
            Unit::Pace(value) => ((value * 30.0) * dpi) as i32,
            Unit::BeardFortnight(value) => ((value * 0.6048 * 0.393701) * dpi) as i32,
        }
    }
}

impl TryFrom<String> for Unit {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let value = value.trim();
        let lower = value.to_lowercase();

        macro_rules! parse_unit {
            ($inp:ident, $suffix:literal, $variant:ident, $type:ty) => {
                if let Some(num) = $inp.strip_suffix($suffix) {
                    return num
                        .trim()
                        .parse::<$type>()
                        .map(Unit::$variant)
                        .map_err(|_| format!("Invalid value for unit '{}': '{}'", $suffix, num));
                }
            };
        }

        parse_unit!(lower, "px", Px, i32);
        parse_unit!(lower, "mm", MM, f32);
        parse_unit!(lower, "cm", CM, f32);
        parse_unit!(lower, "m", M, f32);
        parse_unit!(lower, "in", In, f32);
        parse_unit!(lower, "twip", Twip, f32);
        parse_unit!(lower, "mil", Mil, f32);
        parse_unit!(lower, "pt", Point, f32);
        parse_unit!(lower, "pica", Pica, f32);
        parse_unit!(lower, "ft", Foot, f32);
        parse_unit!(lower, "yd", Yard, f32);
        parse_unit!(lower, "lk", Link, f32);
        parse_unit!(lower, "rd", Rod, f32);
        parse_unit!(lower, "ch", Chain, f32);
        parse_unit!(lower, "ln", Line, f32);
        parse_unit!(lower, "bc", BarleyCorn, f32);
        parse_unit!(lower, "nl", Nail, f32);
        parse_unit!(lower, "fg", Finger, f32);
        parse_unit!(lower, "sk", Stick, f32);
        parse_unit!(lower, "pm", Palm, f32);
        parse_unit!(lower, "sf", Shaftment, f32);
        parse_unit!(lower, "sp", Span, f32);
        parse_unit!(lower, "qr", Quarter, f32);
        parse_unit!(lower, "pc", Pace, f32);
        parse_unit!(lower, "bf", BeardFortnight, f32);

        Err(format!("Unsupported unit or format: '{}'", value))
    }
}
