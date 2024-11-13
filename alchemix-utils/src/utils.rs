use std::hash::{DefaultHasher, Hash, Hasher};

use uuid::Uuid;

pub fn new_uuid() -> String{
    format!("{}", Uuid::new_v4())
}

pub fn hash_str(text: &str) -> String {
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    let hash_value = hasher.finish();
    format!("{:x}", hash_value)
}
