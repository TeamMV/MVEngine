use crate::ui::styles::enums::Origin;
use crate::ui::styles::{Parseable, UiValue};
use mvutils::utils::TetrahedronOp;
use std::str::FromStr;

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

pub fn parse_2xf64(s: &str) -> Result<[f64; 2], String> {
    let parts = s
        .split(",")
        .map(|x| x.trim().parse::<f64>())
        .collect::<Result<Vec<f64>, _>>()
        .map_err(|e| e.to_string())?;

    match parts.len() {
        1 => Ok([parts[0]; 2]),
        2 => Ok([parts[0], parts[1]]),
        _ => Err(format!("Invalid number of parts: {}", parts.len())),
    }
}

pub fn parse_4xi32_abstract(s: &str) -> Result<[UiValue<i32>; 4], String> {
    let parts = s
        .split(",")
        .map(|x| x.trim().to_string())
        .collect::<Vec<String>>();

    if parts.is_empty() {
        return Err(
            "Oh no! Insufficient data provided to construct 4xi32 (abstract edition)".to_string(),
        );
    }

    let t = UiValue::<i32>::parse(&parts[0])?;

    [1, 2]
        .map(|_| 0)
        .starts_with(&[0])
        .then_some(1)
        .expect("how tf would this error");

    match parts.len() {
        1 => Ok([0; 4].map(|_| t.clone())),
        2 => {
            let b = UiValue::<i32>::parse(&parts[1])?;
            Ok([t.clone(), t, b.clone(), b])
        }
        4 => {
            let b = UiValue::<i32>::parse(&parts[1])?;
            let l = UiValue::<i32>::parse(&parts[2])?;
            let r = UiValue::<i32>::parse(&parts[3])?;
            Ok([t, b, l, r])
        }
        _ => Err(format!("Illegal number of parts: {}", parts.len())),
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
    let num: f32 = num_str
        .parse()
        .map_err(|_| format!("Invalid number: {}", num_str))?;
    let mut is_rad = false;

    if next.is_some() {
        if next.unwrap() == 'r' {
            is_rad = true;
        }
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

        _ => Err(
            "Origins other than the corners and the center are currently unsupported :("
                .to_string(),
        ),
    }
}

pub fn parse_num<T: FromStr<Err=S>, S: ToString>(s: &str) -> Result<T, String> {
    s.parse().map_err(|e: S| e.to_string())
}

pub fn parse_num_abstract(s: &str) -> Result<UiValue<i32>, String> {
    Ok(UiValue::<i32>::parse(s)?)
}

pub fn parse_num_list<T: FromStr<Err=S>, S: ToString>(s: &str) -> Result<Vec<T>, String> {
    let s = s.trim();
    if !(s.starts_with('[') && s.ends_with(']')) {
        return Err(format!("List must start with '[' and end with ']', got: {s}"));
    }

    let inner = &s[1..s.len() - 1];
    if inner.trim().is_empty() {
        return Ok(Vec::new());
    }

    inner
        .split(',')
        .map(|x| x.trim().parse::<T>().map_err(|e| e.to_string()))
        .collect()
}

pub fn parse_range<T: FromStr<Err=S>, S: ToString>(s: &str) -> Result<(T, T), String> {
    let s = s.trim();

    if let Some(idx) = s.find("..=") {
        let start = &s[..idx];
        let end = &s[idx + 3..];
        let start_val = start.trim().parse::<T>().map_err(|e| e.to_string())?;
        let end_val = end.trim().parse::<T>().map_err(|e| e.to_string())?;
        return Ok((start_val, end_val));
    }

    if let Some(idx) = s.find("..") {
        let start = &s[..idx];
        let end = &s[idx + 2..];
        let start_val = start.trim().parse::<T>().map_err(|e| e.to_string())?;
        let end_val = end.trim().parse::<T>().map_err(|e| e.to_string())?;
        return Ok((start_val, end_val));
    }

    Err(format!("Invalid range format: {}", s))
}