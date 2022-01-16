use serde::{Deserialize, Serialize};

use super::Component;

#[derive(Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct Content {
    pub schema: Component,
}

impl Content {
    pub fn new<I: Into<Component>>(schema: I) -> Self {
        Self {
            schema: schema.into(),
        }
    }
}
