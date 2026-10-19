use std::{
    cell::RefCell,
    collections::HashMap,
    hash::{BuildHasher, Hasher},
    rc::Rc,
};

use crate::{LispErr, LispExp};

/// environment: like an address book(key->value)
#[derive(Clone, Debug, PartialEq, Default)]
pub struct LispEnv {
    pub data: HashMap<u64, LispExp, BuildFxHasher>,
    pub outer: Option<Rc<RefCell<LispEnv>>>,
}

impl LispEnv {
    pub fn new() -> Self {
        Self {
            data: HashMap::default(),
            outer: None,
        }
    }
    pub fn set(&mut self, key: u64, value: LispExp) {
        self.data.insert(key, value);
    }
    pub fn get(&self, key: u64) -> Result<LispExp, LispErr> {
        if let Some(var) = self.data.get(&key) {
            Ok(var.clone())
        } else if let Some(outer) = &self.outer {
            outer.borrow().get(key)
        } else {
            Err(LispErr::Reason(format!("Undefined variable: {}", key)))
        }
    }
    pub fn with_outer(outer: Rc<RefCell<LispEnv>>) -> Self {
        LispEnv {
            data: HashMap::default(),
            outer: Some(outer),
        }
    }
}

/// implement a personal hasher(FxHasher)
pub struct FxHasher {
    hash: u64,
}

impl FxHasher {
    fn new() -> Self {
        FxHasher { hash: 0 }
    }
    fn write_u64(&mut self, i: u64) {
        self.hash = self
            .hash
            .wrapping_add(i)
            .wrapping_add(0x9e3779b97f4a7c15) // add golden ratio
            .rotate_left(5)
            .wrapping_mul(0x9e3779b97f4a7c15); // multiply by golden ratio
    }
}

impl Hasher for FxHasher {
    fn finish(&self) -> u64 {
        self.hash
    }

    fn write(&mut self, bytes: &[u8]) {
        for chunk in bytes.chunks(8) {
            let mut buf = [0u8; 8];
            buf[..chunk.len()].copy_from_slice(chunk);
            self.write_u64(u64::from_le_bytes(buf));
        }
    }
}

/// build type -- lets hashmap create FxHasher instances
#[derive(Clone, Default)]
pub struct BuildFxHasher;

impl BuildHasher for BuildFxHasher {
    type Hasher = FxHasher;

    fn build_hasher(&self) -> Self::Hasher {
        FxHasher::new()
    }
}
