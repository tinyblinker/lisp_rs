use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{LispErr, LispExp};

/// environment: like an address book(key->value)
#[derive(Clone, Debug, PartialEq, Default)]
pub struct LispEnv {
    pub data: HashMap<String, LispExp>,
    pub outer: Option<Rc<RefCell<LispEnv>>>,
}

impl LispEnv {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            outer: None,
        }
    }
    pub fn set(&mut self, key: String, value: LispExp) {
        self.data.insert(key, value);
    }
    pub fn get(&self, key: &str) -> Result<LispExp, LispErr> {
        if let Some(var) = self.data.get(key) {
            Ok(var.clone())
        } else if let Some(outer) = &self.outer {
            outer.borrow().get(key)
        } else {
            Err(LispErr::Reason(format!("Undefined variable: {}", key)))
        }
    }
    pub fn with_outer(outer: Rc<RefCell<LispEnv>>) -> Self {
        LispEnv {
            data: HashMap::new(),
            outer: Some(outer),
        }
    }
}
