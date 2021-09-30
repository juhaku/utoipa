use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[non_exhaustive]
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Security {
    #[serde(flatten)]
    pub value: HashMap<String, Vec<String>>,
}

impl Security {
    pub fn new() -> Self {
        Default::default()
    }
}
