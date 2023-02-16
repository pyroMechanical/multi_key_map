use core::hash::Hash;
use std::borrow::Borrow;
use std::collections::HashMap;

use crate::entry::{Entry, OccupiedEntry, VacantEntry};

pub mod entry;
pub mod iter;
#[cfg(test)]
mod tests;

//u128 allows us to not store freed indices and keep removal O(1);
//as many as 10 trillion inserts/removes per second
//would still take ~10^18 years to use up the available index space
#[derive(Hash, PartialEq, Eq, Clone, Copy)]
struct Index(u128);

#[derive(Clone)]
/// A wrapper over [HashMap] that allows for multiple keys to point to a single element,
/// providing some additional utilities to make working with multiple keys easier.
pub struct MultiKeyMap<K, V>
where
    K: Hash + Eq,
{
    /// A wrapper over [HashMap] that allows for multiple keys to point to a single element,
    /// providing some additional utilities to make working with multiple keys easier.
    keys: HashMap<K, Index>,
    data: HashMap<Index, (usize, V)>,
    max_index: Index,
}

impl<K, V> Default for MultiKeyMap<K, V> where K: Hash + Eq {
    fn default() -> Self {
        MultiKeyMap {
            keys: HashMap::new(),
            data: HashMap::new(),
            max_index: Index(0),
        }
    }
}
#[allow(dead_code)]
impl<K, V> MultiKeyMap<K, V>
where
    K: Hash + Eq,
{
    ///Creates an empty [MultiKeyMap].
    pub fn new() -> Self {Default::default()}

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
                *count -= 1;
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
    ///Attempts to add a new key to the element at `k`. Returns the new key if `k` is not
    /// in the map.
    pub fn alias<Q>(&mut self, k: &Q, alias: K) -> Result<&mut V, K>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        if self.contains_key(k) {
            let idx = *self.keys.get(k).unwrap();
            let (count, v) = self.data.get_mut(&idx).unwrap();
            *count += 1;
            self.keys.insert(alias, idx);
            Ok(v)
        } else {
            Err(alias)
        }
    }
    ///Attempts to add multiple new keys to the element at `k`. Returns the list of keys if `k` is not
    /// in the map.
    pub fn alias_many<Q>(&mut self, k: &Q, aliases: Vec<K>) -> Result<&mut V, Vec<K>>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        if self.contains_key(k) {
            let idx = *self.keys.get(k).unwrap();
            let (count, v) = self.data.get_mut(&idx).unwrap();
            for alias in aliases {
                *count += 1;
                self.keys.insert(alias, idx);
            }
            Ok(v)
        } else {
            Err(aliases)
        }
    }
    ///An iterator visiting all keys in an arbitrary order. Equivalent to [HashMap]::[`keys`].
    /// 
    /// [`keys`]: HashMap::keys
    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.keys.keys()
    }
    ///An iterator visiting all elements in an arbitrary order. Equivalent to [HashMap]::[`values`].
    /// 
    /// [`values`]: HashMap::values
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.data.values().map(|(_, v)| v)
    }
    ///An iterator visiting all elements in an arbitrary order, while allowing mutation. Equivalent to [HashMap]::[`values_mut`].
    /// 
    /// [`values_mut`]: HashMap::values_mut
    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.data.values_mut().map(|(_, v)| v)
    }
    ///Consumes the map and provides an iterator over all keys. Equivalent to [HashMap]::[`into_keys`].
    /// 
    /// [`into_keys`]: HashMap::into_keys
    pub fn into_keys(self) -> impl Iterator<Item = K> {
        self.keys.into_keys()
    }
    ///Consumes the map and provides an iterator over all values. Equivalent to [HashMap]::[`into_values`].
    /// 
    /// [`into_values`]: HashMap::into_values
    pub fn into_values(self) -> impl Iterator<Item = V> {
        self.data.into_values().map(|(_, v)| v)
    }
    ///An iterator visiting all key-value pairs. Due to the nature of [MultiKeyMap], value references
    /// may be shared across multiple keys.
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        iter::Iter::new(self)
    }
    ///Returns a shared reference to the value of the key. Equivalent to [HashMap]::[`get`].
    /// 
    /// [`get`]: HashMap::get
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
    ///Returns a mutable reference to the value of the key. Equivalent to [HashMap]::[`get_mut`].
    /// 
    /// [`get_mut`]: HashMap::get_mut
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
    ///inserts a new value, pairs it to a list of keys, and returns the values that existed
    /// at each key if there are no other keys to that value.
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
                    if let Some((_, v)) = self.data.remove(idx) {
                        bumped.push(v);
                    }
                } else {
                    *count -= 1;
                    new_count += 1;
                    self.keys.insert(k, new_idx);
                }
            } else {
                new_count += 1;
                self.keys.insert(k, new_idx);
            }
        }
        self.data.insert(new_idx, (new_count, v));
        bumped
    }
    ///Removes a key from the map, returning the value at that key if it existed in the map
    /// and no other keys share that value.
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
                *count -= 1;
                self.keys.remove(k);
                None
            }
        } else {
            None
        }
    }
    ///Removes a list of keys from the map, returning the values at each key if they existed in the map
    /// and no other keys shared that value.
    pub fn remove_many<Q>(&mut self, ks: &[Q]) -> Vec<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let mut bumped = vec![];
        for k in ks {
            if let Some(v) = self.remove(k) {
                bumped.push(v);
            }
        }
        bumped
    }
    ///Equivalent to [HashMap]::[`entry`].
    /// 
    /// [`entry`]: HashMap::entry
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
