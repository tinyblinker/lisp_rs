use std::collections::HashMap;

use crate::{LispErr, LispExp};

/// environment: like an address book(key->value)
#[derive(Clone, Debug, PartialEq)]
pub struct LispEnv {
    pub data: HashMap<String, LispExp>,
}

impl LispEnv {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
    pub fn set(&mut self, key: String, value: LispExp) {
        self.data.insert(key, value);
    }
    pub fn get(&self, key: &str) -> Result<LispExp, LispErr> {
        self.data
            .get(key)
            .cloned()
            .ok_or_else(|| LispErr::Reason(format!("Undefined variable: {}", key)))
    }
}
