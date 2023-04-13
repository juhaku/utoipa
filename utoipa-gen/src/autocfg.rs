use std::sync::RwLock;

use once_cell::sync::Lazy;

pub struct Schema {
    name: String,
    schema: String,
    registered: bool,
}

impl Schema {
    pub fn new(name: String, schema: String) -> Self {
        Self {
            name,
            schema,
            registered: false,
        }
    }
}

pub static SCHEMAS: Lazy<RwLock<Vec<Schema>>> = Lazy::new(|| RwLock::new(vec![]));
