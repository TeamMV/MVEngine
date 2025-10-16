use itertools::Itertools;

#[derive(Clone, Debug)]
pub enum Base64DecodeError {
    IllegalCharacter(char),
    IllegalPadding
}

fn char_to_num(c: char) -> Option<u8> {
    match c {
        'A'..='Z' => Some((c as u8) - b'A'),
        'a'..='z' => Some((c as u8) - b'a' + 26),
        '0'..='9' => Some((c as u8) - b'0' + 52),
        '+' => Some(62),
        '/' => Some(63),
        _ => None,
    }
}

pub fn decode_base64(base64: &str) -> Result<Vec<u8>, Base64DecodeError> {
    let len = base64.len();
    if len % 4 != 0 {
        return Err(Base64DecodeError::IllegalPadding);
    }

    let mut bytes = Vec::with_capacity(len * 3 / 4);
    for (c1, c2, c3, c4) in base64.chars().tuples() {
        let pad3 = c3 == '=';
        let pad4 = c4 == '=';

        let n1 = char_to_num(c1).ok_or(Base64DecodeError::IllegalCharacter(c1))?;
        let n2 = char_to_num(c2).ok_or(Base64DecodeError::IllegalCharacter(c2))?;
        let n3 = if !pad3 { char_to_num(c3).ok_or(Base64DecodeError::IllegalCharacter(c3))? } else { 0 };
        let n4 = if !pad4 { char_to_num(c4).ok_or(Base64DecodeError::IllegalCharacter(c4))? } else { 0 };

        let b1 = (n1 << 2) | (n2 >> 4);
        bytes.push(b1);

        if !pad3 {
            let b2 = ((n2 & 0b1111) << 4) | (n3 >> 2);
            bytes.push(b2);
        }

        if !pad4 {
            let b3 = ((n3 & 0b11) << 6) | n4;
            bytes.push(b3);
        }
    }

    Ok(bytes)
}