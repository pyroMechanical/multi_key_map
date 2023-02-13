use core::hash::Hash;
use std::borrow::Borrow;
use std::collections::HashMap;

use crate::entry::{Entry, OccupiedEntry, VacantEntry};

mod entry;
#[cfg(test)]
mod tests;

//u128 allows us to not store freed indices and keep removal O(1);
//as many as 10 trillion inserts/removes per second
//would still take ~10^18 years to use up the available index space
#[derive(Hash, PartialEq, Eq, Clone, Copy)]
struct Index(u128);
#[derive(Clone)]
struct MultiKeyMap<K, V>
where
    K: Hash + Eq,
{
    keys: HashMap<K, Index>,
    data: HashMap<Index, (usize, V)>,
    max_index: Index,
}
impl<K, V> MultiKeyMap<K, V>
where
    K: Hash + Eq,
{
    pub fn new() -> Self {
        MultiKeyMap {
            keys: HashMap::new(),
            data: HashMap::new(),
            max_index: Index(0),
        }
    }

    pub(crate) fn next_index(&mut self) -> Index {
        let idx = self.max_index;
        self.max_index = Index(self.max_index.0 + 1);
        idx
    }

    pub fn contains_key<Q>(&self, k: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.keys.contains_key(k)
    }

    ///inserts a new value at a given key, and returns the value at that key if
    /// there are no other keys to that value. otherwise returns [`None`].
    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        if self.contains_key(&k) {
            let idx = self.keys.get(&k).unwrap();
            let (count, _) = self.data.get_mut(idx).unwrap();
            if *count <= 1 {
                self.data.insert(*idx, (1, v)).map(|(_, v)| v)
            } else {
                *count = *count - 1;
                let new_idx = self.max_index;
                self.max_index = Index(self.max_index.0 + 1);
                self.keys.insert(k, new_idx);
                self.data.insert(new_idx, (1, v));
                None
            }
        } else {
            let new_idx = self.max_index;
            self.max_index = Index(self.max_index.0 + 1);
            self.keys.insert(k, new_idx);
            self.data.insert(new_idx, (1, v));
            None
        }
    }

    pub fn alias<Q>(&mut self, k: &Q, alias: K) -> Result<&mut V, K>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        if self.contains_key(k) {
            let idx = *self.keys.get(k).unwrap();
            let (count, v) = self.data.get_mut(&idx).unwrap();
            *count = *count + 1;
            self.keys.insert(alias, idx);
            Ok(v)
        } else {
            Err(alias)
        }
    }

    pub fn alias_many<Q>(&mut self, k: &Q, aliases: Vec<K>) -> Result<&mut V, Vec<K>>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        if self.contains_key(k) {
            let idx = *self.keys.get(k).unwrap();
            let (count, v) = self.data.get_mut(&idx).unwrap();
            for alias in aliases {
                *count = *count + 1;
                self.keys.insert(alias, idx);
            }
            Ok(v)
        } else {
            Err(aliases)
        }
    }

    pub fn get<Q>(&self, k: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.keys
            .get(k)
            .and_then(|idx| self.data.get(idx))
            .map(|(_, v)| v)
    }

    pub fn get_mut<Q>(&mut self, k: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.keys
            .get(k)
            .and_then(|idx| self.data.get_mut(idx))
            .map(|(_, v)| v)
    }

    pub fn insert_many(&mut self, ks: Vec<K>, v: V) -> Vec<V> {
        let mut bumped = vec![];
        let new_idx = self.max_index;
        self.max_index = Index(self.max_index.0 + 1);
        let mut new_count = 0;
        for k in ks {
            if self.contains_key(&k) {
                let idx = self.keys.get(&k).unwrap();
                let (count, _) = self.data.get_mut(idx).unwrap();
                if *count <= 1 {
                    self.data.remove(idx).map(|(_, v)| bumped.push(v));
                } else {
                    *count = *count - 1;
                    new_count = new_count + 1;
                    self.keys.insert(k, new_idx);
                }
            } else {
                new_count = new_count + 1;
                self.keys.insert(k, new_idx);
            }
        }
        self.data.insert(new_idx, (new_count, v));
        bumped
    }

    pub fn remove<Q>(&mut self, k: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        if self.contains_key(k) {
            let idx = self.keys.get(k).unwrap();
            let (count, _) = self.data.get_mut(idx).unwrap();
            if *count == 1 {
                let result = self.data.remove(idx).map(|(_, v)| v);
                self.keys.remove(k);
                result
            } else {
                *count = *count - 1;
                self.keys.remove(k);
                None
            }
        } else {
            None
        }
    }

    pub fn remove_many<Q>(&mut self, ks: &[Q]) -> Vec<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let mut bumped = vec![];
        for k in ks {
            self.remove(k).map(|v| bumped.push(v));
        }
        bumped
    }

    pub fn entry(&mut self, k: K) -> Entry<K, V> {
        if let Some(idx) = self.keys.get(&k) {
            Entry::Occupied(OccupiedEntry {
                key: k,
                idx: *idx,
                map: self,
            })
        } else {
            Entry::Vacant(VacantEntry { key: k, map: self })
        }
    }
}

impl<K, V, const N: usize> From<[(Vec<K>, V); N]> for MultiKeyMap<K, V>
where
    K: Hash + Eq,
{
    fn from(arr: [(Vec<K>, V); N]) -> Self {
        Self::from_iter(arr)
    }
}

impl<K, V> FromIterator<(Vec<K>, V)> for MultiKeyMap<K, V>
where
    K: Hash + Eq,
{
    fn from_iter<T: IntoIterator<Item = (Vec<K>, V)>>(iter: T) -> Self {
        let mut map = Self::new();
        for (keys, value) in iter {
            map.insert_many(keys, value);
        }
        map
    }
}
