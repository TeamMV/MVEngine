use mvutils::utils::TetrahedronOp;
use crate::ui::styles::Origin;

pub fn parse_4xi32(s: &str) -> Result<[i32; 4], String> {
    let parts = s
        .split(",")
        .map(|x| x.trim().parse::<i32>())
        .collect::<Result<Vec<i32>, _>>()
        .map_err(|e| e.to_string())?;

    match parts.len() {
        1 => Ok([parts[0]; 4]),
        2 => Ok([parts[0], parts[0], parts[1], parts[1]]),
        4 => Ok([parts[0], parts[1], parts[2], parts[3]]),
        _ => Err(format!("Invalid number of parts: {}", parts.len())),
    }
}

pub fn parse_angle(s: &str) -> Result<f32, String> {
    let mut num_str = String::new();
    let mut iter = s.chars();
    let mut next = iter.next();
    while let Some(x) = next {
        if x.is_numeric() || x == '.' {
            num_str.push(x);
        } else {
            next = iter.next();
            break;
        }
        next = iter.next();
    }
    let num: f32 = num_str.parse().map_err(|_| format!("Invalid number: {}", num_str))?;
    let mut is_rad = false;

    if next.is_some() {
        if next.unwrap() == 'r' { is_rad = true; }
    }
    Ok(is_rad.yn(num.to_radians(), num))
}

pub fn parse_origin(s: &str) -> Result<Origin, String> {
    match s {
        "center" => Ok(Origin::Center),
        "top_left" => Ok(Origin::TopLeft),
        "top_right" => Ok(Origin::TopRight),
        "bottom_left" => Ok(Origin::BottomLeft),
        "bottom_right" => Ok(Origin::BottomRight),

        _ => {
            Err("Origins other than the corners and the center are currently unsupported :(".to_string())
        }
    }
}