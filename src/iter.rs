use crate::{Index, MultiKeyMap};
use std::collections::HashMap;
use std::hash::Hash;
use std::iter::Iterator;

#[allow(dead_code)]
///Immutable iterator over each key-value pair. see [`iter`] on [`MultiKeyMap`]
/// for more information.
///
/// [`iter`]: MultiKeyMap::iter
pub struct Iter<'a, K, V>
where
    K: Hash + Eq,
{
    keys: std::collections::hash_map::Iter<'a, K, Index>,
    map: &'a HashMap<Index, (usize, V)>,
}

impl<'a, K, V> Iter<'a, K, V>
where
    K: Hash + Eq,
{
    pub fn new(map: &'a MultiKeyMap<K, V>) -> Self {
        let keys = map.keys.iter();
        Self {
            keys,
            map: &map.data,
        }
    }
}

impl<'a, K, V> Iterator for Iter<'a, K, V>
where
    K: Hash + Eq,
{
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        match self.keys.next() {
            None => None,
            Some((k, idx)) => self.map.get(idx).map(|(_, v)| (k, v)),
        }
    }
}
