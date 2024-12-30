//! this module implements a linked hashmap
use std::{
    borrow::{Borrow}, hash::{DefaultHasher, Hash, Hasher}, mem
};
const INITIAL_NBUCKET: usize = 1;

pub struct Hashmap<K, V> {
    buckets: Vec<Vec<(K, V)>>,
    items: usize,
}

pub struct Iter<'a, K, V> {
    map: &'a Hashmap<K, V>,
    current_bucket: usize,
    current_item: usize,
}

pub enum Entry<'a, K, V> {
    Occupied(OccupiedEntry<'a, K, V>),
    Vacant(VacantEntry<'a, K, V>),
}

#[allow(dead_code)]
pub struct OccupiedEntry<'a, K, V> {
    element: &'a mut (K, V),
}

#[allow(dead_code)]
pub struct VacantEntry<'a, K, V> {
    bucket: &'a mut Vec<(K, V)>,
    key: K,
}

impl<'a, K, V> VacantEntry<'a, K, V> {
    pub fn insert(self, default: V) -> &'a mut V {
        self.bucket.push((self.key, default));
        // unwrap in this case is safe because we've just pushed the element
        // we know, it's there!
        &mut self.bucket.last_mut().unwrap().1
    }
}


impl<'a, K, V> Entry<'a, K, V> {
    pub fn insert_or(self, default: V) -> &'a mut V {
        match self {
            Entry::Occupied(e) => &mut e.element.1,
            Entry::Vacant(e) => e.insert(default),
        }
    }

    pub fn or_default(self) -> &'a mut V 
    where 
        V: Default,
    {
        match self {
            Entry::Occupied(e) => &mut e.element.1,
            Entry::Vacant(e) => e.insert(V::default()),
        }
    }
}


impl<'a, K, V> Iter<'a, K, V> {
    fn new(map: &'a Hashmap<K, V>) -> Self{
        Iter {
            map,
            current_bucket : 0,
            current_item : 0,
        }
    }
}


impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.map.buckets.get(self.current_bucket) {
                Some(bucket) => match bucket.get(self.current_item) {
                    Some(&(ref key, ref val)) => {
                        self.current_item += 1;
                        break Some((key, val));
                    }
                    None => {
                        self.current_bucket += 1;
                        self.current_item = 0;
                        continue;
                    }
                },
                None => break None,
            }
        }

    }
}

impl<'a, K, V> IntoIterator for &'a Hashmap<K, V> {
    type IntoIter = Iter<'a, K, V>;
    type Item = (&'a K, &'a V);

    fn into_iter(self) -> Self::IntoIter {
        Iter::new(self)
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
where
    K: Eq + Hash,
{
    fn bucket<Q>(&self, key: &Q) -> usize 
    where 
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
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

    pub fn resize(&mut self) {
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

    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where 
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.buckets[self.bucket(key)]
            .iter()
            .find(|(ref ekey, _)| ekey.borrow() == key)
            .map(|(_, ref eval)| eval)
    }

    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where 
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.buckets[self.bucket(key)]
            .iter()
            .any(|(ref ekey, _)| ekey.borrow() == key)
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where 
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let bucket = self.bucket(key);
        let bucket = &mut self.buckets[bucket];
        // ? works with both result and option (sugar)
        let ind: usize = bucket.iter().position(|(ref ekey, _)| ekey.borrow() == key)?;
        self.items -= 1;
        Some(bucket.swap_remove(ind).1)
    }


    pub fn entry<'a>(&'a mut self, key: K) -> Entry<'a, K, V> {
        if self.buckets.is_empty() || self.items > 3 * self.buckets.len() / 4 {
            self.resize();
        }
        
        let bucket = self.bucket(&key);
        match self.buckets[bucket].iter().position(|&(ref ekey, _)| ekey == &key) {
            Some(index) => Entry::Occupied(OccupiedEntry {
                element: &mut self.buckets[bucket][index]
            }),
            None => Entry::Vacant(VacantEntry {
                bucket: &mut self.buckets[bucket],
                key
            })
        }
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
        assert_eq!(map.remove(&"foobar"), Some(3));
        assert_eq!(map.remove(&"foobar"), None);
    }

    #[test]
    fn iter() {
        let mut map = Hashmap::new();
        map.insert("foo", 1);
        map.insert("bar", 2);
        map.insert("foobar", 3);
        for (&k, &v) in &map {
            match k {
                "foo" => assert_eq!(v, 1),
                "bar" => assert_eq!(v, 2),
                "foobar" => assert_eq!(v, 3),
                _ => unreachable!(),
            }
        }
        assert_eq!((&map).into_iter().count(), 3);
    }
}
