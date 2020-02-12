use std::collections::{BTreeMap,HashMap};
use std::sync::{Arc, RwLock};
use super::{Key, Value};

#[derive(Clone)]
pub struct Engine {
    btree: Arc<RwLock<BTreeMap<String, String>>>,
}

impl Engine {
	pub fn new() -> Self{
		Engine {
			btree: Arc::new(RwLock::new(BTreeMap::new()))
		}
	}
	
    pub fn get(&self, k: Key) -> Result<Option<String>,()> {
		let data = self.btree.read().unwrap();
        let ret = data.get(&k);
        match ret {
            Some(s) => Ok(Some(s.clone())),
            None => Ok(None)
        }
    }

	pub fn put(&mut self, k: Key, v: Value) -> Result<bool,()> {
		let mut data = self.btree.write().unwrap();
        let ret = data.insert(k, v);
		match ret {
            Some(_) => Ok(true),
            None => Ok(false)
        }
    }

    pub fn delete(&mut self, k: Key) -> Result<Option<String>,()> {		
		let mut data = self.btree.write().unwrap();
		let ret = data.remove(&k);
		match ret {
            Some(s) => Ok(Some(s.clone())),
            None => Ok(None)
        }
    }

    pub fn scan(&self, k1: Key, k2: Key) -> Result<Option<HashMap<String,String>>,()> {
		let data = self.btree.read().unwrap();
        let range = data.range(k1..k2);
        let mut hmap = HashMap::new();
        for (k, v) in range {
            hmap.insert(k.clone(),v.clone());
        }
        match hmap.len() {
            0 => Ok(None),
            _ => Ok(Some(hmap))
        }
    }

}

