// Live scope ka byte budget - interpreter aur VM dono yahi use karte hain
// Ek scope ka byte hisaab: variables ke *stored* values ka total size

use std::collections::HashMap;

use crate::interpreter::value::Value;

pub(crate) struct Scope {
    pub(crate) map: HashMap<String, Value>,
    pub(crate) bytes: u64,
}

impl Scope {
    pub(crate) fn new() -> Self {
        Scope {
            map: HashMap::new(),
            bytes: 0,
        }
    }

    pub(crate) fn total_bytes_add(&mut self, delta: i128) {
        if delta >= 0 {
            self.bytes += delta as u64;
        } else {
            self.bytes = self.bytes.saturating_sub((-delta) as u64);
        }
    }
}
