pub fn decode(data: Vec<u8>) -> Vec<u8> {
    data
}

pub fn encode(data: Vec<u8>) -> Vec<u8> {
    let len = data.len() as u32;
    let mut new_data = len.to_le_bytes().to_vec();
    new_data.extend(data);
    new_data
}