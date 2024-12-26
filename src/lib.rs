//! this module implements a linked hashmap

use std::{hash::{DefaultHasher, Hash, Hasher}, mem};
const INITIAL_NBUCKET: usize = 1;

pub struct Hashmap<K, V> {
    bucket: Vec<Vec<(K, V)>>,
    items: usize,
}

impl<K, V> Hashmap<K, V> {
    pub fn new() -> Self {
        Hashmap { 
            bucket: Vec::new(),
            items: 0,
        }
    }
}

impl<K, V> Default for Hashmap<K, V> {
     fn default() -> Self {
         Self::new()
     }
} 

impl<K, V> Hashmap<K, V>
where K: Eq + Hash 
{
    fn bucket(&self, key: &K) -> usize {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        (hasher.finish() & (self.bucket.len() - 1) as u64) as usize
    }
    
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if self.bucket.is_empty() || self.items > 3 * self.bucket.len() / 4 {
            self.resize();
        }

        let bucket = self.bucket(&key); 
        let bucket = &mut self.bucket[bucket];
        
        // ref, we don't wanna take ownership (otherwise we'll break the data structure)
        for &mut (ref ekey, ref mut eval) in bucket.iter_mut() {
            if ekey == &key {
                return Some(mem::replace(eval, value));
            }
        }
        self.items += 1;
        bucket.push((key, value));
        None
    } 

    pub fn resize(&mut self){
        let target_size = match self.bucket.len() {
            0 => INITIAL_NBUCKET,
            n => 2 * n,
        };

        let mut new_bucket = Vec::with_capacity(target_size);
        new_bucket.extend((0..target_size).map(|_| Vec::new()));
         
        for (key, value) in self.bucket.iter_mut().flat_map(|bucket| bucket.drain(..)) {
            let mut hasher = DefaultHasher::new();
            key.hash(&mut hasher);
            let bucket: usize = (hasher.finish() & (new_bucket.len() - 1) as u64) as usize;
            new_bucket[bucket].push((key, value));
        }
        self.bucket = new_bucket;
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.bucket[self.bucket(key)]
            .iter()
            .find(|(ref ekey, _)| ekey == key)
            .map(|(_, ref eval)| eval)
    }


    pub fn contains_key(&self, key: &K) -> bool {
        self.bucket[self.bucket(key)]
            .iter()
            .any(|(ref ekey, _)| ekey == key)
    }

    pub fn remove(&mut self, key: K) -> Option<V> {
        let bucket = self.bucket(&key);
        let bucket = &mut self.bucket[bucket];
        // ? works with both result and option (sugar)
        let ind: usize = bucket.iter().position(|(ref ekey, _)| ekey == &key)?;
        self.items -= 1;
        Some(bucket.swap_remove(ind).1)
    }

    pub fn is_empty(&self) -> bool {
        self.items == 0
    }

    pub fn len(&self) -> usize {
        self.items
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert() {
        let mut map = Hashmap::new();
        assert_eq!(map.len(), 0);
        assert!(map.is_empty());
        map.insert("foo", 1);
        assert_eq!(map.len(), 1);
        assert!(!map.is_empty());
        assert_eq!(map.get(&"foo"), Some(&1));
    }

    #[test]
    fn remove() {
        let mut map = Hashmap::new();
        map.insert("foo", 1);
        map.insert("bar", 2);
        map.insert("foobar", 3);
        assert_eq!(map.remove("foobar"), Some(3));
        assert_eq!(map.remove("foobar"), None);
    }

}

