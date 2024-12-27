//! this module implements a linked hashmap

use std::{hash::{DefaultHasher, Hash, Hasher}, mem};
const INITIAL_NBUCKET: usize = 1;

pub struct Hashmap<K, V> {
    buckets: Vec<Vec<(K, V)>>,
    items: usize,
}

pub struct IterHashmap<'a, K, V> {
    map: &'a Hashmap<K, V>,
    current_bucket: usize,
    current_item: usize,
}
    
impl<'a, K, V> Iterator for IterHashmap<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.map.buckets.get(self.current_bucket) {
                Some(bucket) => {
                    match bucket.get(self.current_item) {
                        Some((ref key, ref val)) => {
                            break Some((key, val))
                        }
                        None => {
                            continue;
                        }
                    }
                }
                None => break None,
            }
        }
    } 
}


impl<'a, K, V> IntoIterator for &'a Hashmap<K, V> {
    type IntoIter = IterHashmap<'a, K, V>;
    type Item = (&'a K, &'a V);

    fn into_iter(self) -> Self::IntoIter {
        IterHashmap {
            map: self,
            current_bucket: 0,
            current_item: 0,
        }
    }
}

impl<K, V> Hashmap<K, V> {
    pub fn new() -> Self {
        Hashmap { 
            buckets: Vec::new(),
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
        (hasher.finish() & (self.buckets.len() - 1) as u64) as usize
    }
    
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if self.buckets.is_empty() || self.items > 3 * self.buckets.len() / 4 {
            self.resize();
        }

        let bucket = self.bucket(&key); 
        let bucket = &mut self.buckets[bucket];
        
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
        let target_size = match self.buckets.len() {
            0 => INITIAL_NBUCKET,
            n => 2 * n,
        };

        let mut new_bucket = Vec::with_capacity(target_size);
        new_bucket.extend((0..target_size).map(|_| Vec::new()));
         
        for (key, value) in self.buckets.iter_mut().flat_map(|bucket| bucket.drain(..)) {
            let mut hasher = DefaultHasher::new();
            key.hash(&mut hasher);
            let bucket: usize = (hasher.finish() & (new_bucket.len() - 1) as u64) as usize;
            new_bucket[bucket].push((key, value));
        }
        self.buckets = new_bucket;
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.buckets[self.bucket(key)]
            .iter()
            .find(|(ref ekey, _)| ekey == key)
            .map(|(_, ref eval)| eval)
    }


    pub fn contains_key(&self, key: &K) -> bool {
        self.buckets[self.bucket(key)]
            .iter()
            .any(|(ref ekey, _)| ekey == key)
    }

    pub fn remove(&mut self, key: K) -> Option<V> {
        let bucket = self.bucket(&key);
        let bucket = &mut self.buckets[bucket];
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

