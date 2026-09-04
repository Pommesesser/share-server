use rand::RngExt;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};

const N: usize = 8;

pub fn gen_rand_id() -> String {
    let mut bytes = [0u8; N];
    rand::rng().fill(&mut bytes);

    URL_SAFE_NO_PAD.encode(bytes)
}
