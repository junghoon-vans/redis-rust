use std::collections::HashMap;
use std::time::Instant;
#[derive(Debug)]
pub struct Item {
    pub value: String,
    pub created: Instant,
    pub expires: usize,
}
pub struct Storage {
    pub storage: HashMap<String, Item>,
}
impl Storage {
    pub fn new() -> Self {
        Storage {
            storage: HashMap::new(),
        }
    }
    pub fn set(&mut self, key: &str, value: &str, expires: usize) {
        let item = Item {
            value: String::from(value),
            created: Instant::now(),
            expires,
        };
        self.storage.insert(String::from(key), item);
    }
    pub fn get(&self, key: &str) -> Option<&Item> {
        let item = self.storage.get(key)?;
        
        let is_expired =
            item.expires > 0 && item.created.elapsed().as_millis() > item.expires as u128;
      
        match is_expired {
            true => None,
            false => Some(item),
        }
    }
}
impl Default for Storage {
    fn default() -> Self {
        Storage::new()
    }
}
