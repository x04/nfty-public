use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn epoch_time() -> Duration {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
}

pub fn decode_hex(v: &str) -> Result<Vec<u8>, hex::FromHexError> {
    if v.len() >= 2 && &v[0..2] == "0x" {
        hex::decode(&v[2..])
    } else {
        hex::decode(&v)
    }
}
