use std::convert::Infallible;

use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use hyper::body::Bytes;
use rand::{Rng, SeedableRng};

pub fn generate_random_string(len: usize) -> String {
    let timestamp = chrono::Utc::now().timestamp();
    let mut rng = rand::rngs::StdRng::seed_from_u64(timestamp as u64);
    let chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let random_string: String = (0..len)
        .map(|_| {
            let index = rng.gen_range(0..chars.len());
            chars.chars().nth(index).unwrap()
        })
        .collect();
    random_string
}

pub fn ip_string_to_array(ip_string: &str) -> Option<[u8; 4]> {
    let parts: Vec<&str> = ip_string.split('.').collect();

    if parts.len() == 4 {
        let mut result = [0; 4];
        for (i, part) in parts.iter().enumerate() {
            if let Ok(num) = part.parse::<u8>() {
                result[i] = num;
            } else {
                return None; // Parsing failed
            }
        }
        Some(result)
    } else {
        None // Invalid IP address format
    }
}

pub fn serve_empty() -> BoxBody<Bytes, Infallible> {
    Empty::<Bytes>::new().boxed()
}

pub fn serve_full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, Infallible> {
    Full::new(chunk.into()).boxed()
}
