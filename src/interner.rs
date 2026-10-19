use std::{
    collections::HashMap,
    sync::{OnceLock, RwLock},
};

struct Interner {
    str_to_id: HashMap<String, u64>,
    id_to_str: Vec<String>,
    total_count: u64,
}

impl Interner {
    pub fn new() -> Self {
        Interner {
            str_to_id: HashMap::new(),
            id_to_str: Vec::new(),
            total_count: 0,
        }
    }

    pub fn intern(&mut self, s: &str) -> u64 {
        if let Some(&id) = self.str_to_id.get(s) {
            id
        } else {
            let id = self.id_to_str.len() as u64;
            self.str_to_id.insert(s.to_string(), id);
            self.id_to_str.push(s.to_string());
            self.total_count += 1;
            id
        }
    }
}

static INTERNER: OnceLock<RwLock<Interner>> = OnceLock::new();

pub fn intern(s: &str) -> u64 {
    let mut interner = INTERNER
        .get_or_init(|| RwLock::new(Interner::new()))
        .write()
        .unwrap();
    let id = interner.intern(s);
    id
}
