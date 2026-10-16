use std::{
    collections::HashMap,
    sync::{OnceLock, RwLock},
};

struct Interner {
    strings: HashMap<String, u64>,
    counter: u64,
}

impl Interner {
    pub fn new() -> Self {
        Interner {
            strings: HashMap::new(),
            counter: 0,
        }
    }

    pub fn intern(&mut self, s: &str) -> u64 {
        if let Some(&id) = self.strings.get(s) {
            id
        } else {
            let id = self.counter.clone();
            self.strings.insert(s.to_string(), id);
            self.counter = self.counter + 1;
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
