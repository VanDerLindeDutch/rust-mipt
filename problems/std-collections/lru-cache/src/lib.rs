#![forbid(unsafe_code)]

use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;
use std::time::Instant;

#[derive(Debug)]
pub struct LRUCache<K, V> {
    capacity: usize,
    currentlyUsed: usize,
    lastUsed: BTreeMap<Instant, K>,
    used: BTreeMap<K, Instant>,
    map: HashMap<K, V>,

}
impl<K: Clone + Hash + Ord, V> LRUCache<K, V> {
    pub fn new(capacity: usize) -> Self {
        if capacity == 0 {
            panic!()
        }
        // TODO: your code goes here.
        Self {
            lastUsed: BTreeMap::new(),
            used: BTreeMap::new(),
            capacity,
            currentlyUsed: 0,
            map: HashMap::with_capacity(capacity),
        }
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        let q = *self.map;
        self.updateTime(key.clone());
        // TODO: your code goes here.
        self.map.get(key)
    }
    fn updateTime(&mut self, key: K) {
        let time = self.used.remove(&key);
        if time.is_none(){
            return;
        }
        let borrowedKey = self.lastUsed.remove(&time.unwrap()).unwrap();
        let now = Instant::now();
        self.lastUsed.insert(now, borrowedKey);
        self.used.insert(key, now);
    }
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {

        if self.map.contains_key(&key) {
            self.updateTime(key.clone());
            return self.map.insert(key, value);
        }
        match &self.currentlyUsed.cmp(&self.capacity){
            Ordering::Less => {
                let now = Instant::now();
                self.used.insert(key.clone(), now);
                self.lastUsed.insert(now, key.clone());
                self.currentlyUsed+=1;
            }
            Ordering::Equal => {
                let now = Instant::now();
                let toDrop = &self.lastUsed.pop_first();
                self.map.remove(&toDrop.as_ref().unwrap().1);
                self.lastUsed.remove(&(toDrop.as_ref().unwrap().0));
                self.used.remove(&(toDrop.as_ref().unwrap().1));
                self.lastUsed.insert(now, key.clone());
                self.used.insert(key.clone(), now);
            }
            Ordering::Greater => {unreachable!()}
        }
        // TODO: your code goes here.
        self.map.insert(key, value)
    }
}
