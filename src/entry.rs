use crate::Index;
use crate::MultiKeyMap;
use std::hash::Hash;

pub enum Entry<'a, K, V>
where
    K: Hash + Eq,
{
    Occupied(OccupiedEntry<'a, K, V>),
    Vacant(VacantEntry<'a, K, V>),
}

impl<'a, K, V> Entry<'a, K, V>
where
    K: Hash + Eq,
{
    pub fn and_modify<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut V),
    {
        match self {
            Self::Vacant(_) => self,
            Self::Occupied(mut entry) => {
                f(entry.get_mut());
                Self::Occupied(entry)
            }
        }
    }

    pub fn or_insert(self, v: V) -> &'a mut V {
        match self {
            Self::Occupied(entry) => entry.into_mut(),
            Self::Vacant(entry) => entry.insert(v),
        }
    }

    pub fn or_insert_with<F>(self, f: F) -> &'a mut V
    where
        F: FnOnce() -> V,
    {
        match self {
            Self::Occupied(entry) => entry.into_mut(),
            Self::Vacant(entry) => entry.insert(f()),
        }
    }

    pub fn or_insert_with_key<F>(self, f: F) -> &'a mut V
    where
        F: FnOnce(&K) -> V,
    {
        match self {
            Self::Occupied(entry) => entry.into_mut(),
            Self::Vacant(entry) => {
                let value = f(entry.key());
                entry.insert(value)
            }
        }
    }

    pub fn key(&self) -> &K {
        match self {
            Self::Occupied(entry) => entry.key(),
            Self::Vacant(entry) => entry.key(),
        }
    }
}

impl<'a, K, V> Entry<'a, K, V>
where
    K: Hash + Eq,
    V: Default,
{
    pub fn or_default(self) -> &'a mut V {
        match self {
            Self::Occupied(entry) => entry.into_mut(),
            Self::Vacant(entry) => entry.insert(Default::default()),
        }
    }
}

pub struct OccupiedEntry<'a, K, V>
where
    K: Hash + Eq,
{
    pub(crate) key: K,
    pub(crate) idx: Index,
    pub(crate) map: &'a mut MultiKeyMap<K, V>,
}

impl<'a, K, V> OccupiedEntry<'a, K, V>
where
    K: Hash + Eq,
{
    pub fn key(&self) -> &K {
        &self.key
    }

    pub fn remove_entry(self) -> (K, Option<V>) {
        let key_ref = &self.key;
        let value = self.map.data.remove(&self.idx).map(|(_, v)| v);
        let key = self.map.keys.remove_entry(key_ref).map(|(k, _)| k).unwrap();
        (key, value)
    }

    pub fn get(&self) -> &V {
        self.map.data.get(&self.idx).map(|(_, v)| v).unwrap()
    }

    pub fn get_mut(&mut self) -> &mut V {
        self.map.data.get_mut(&self.idx).map(|(_, v)| v).unwrap()
    }

    pub fn into_mut(self) -> &'a mut V {
        self.map.data.get_mut(&self.idx).map(|(_, v)| v).unwrap()
    }

    pub fn remove(self) -> Option<V> {
        let key = &self.key;
        self.map.remove(key)
    }
}

pub struct VacantEntry<'a, K, V>
where
    K: Hash + Eq,
{
    pub(crate) key: K,
    pub(crate) map: &'a mut MultiKeyMap<K, V>,
}

impl<'a, K, V> VacantEntry<'a, K, V>
where
    K: Hash + Eq,
{
    fn key(&self) -> &K {
        &self.key
    }
    fn into_key(self) -> K {
        self.key
    }
    fn insert(self, value: V) -> &'a mut V {
        let idx = self.map.next_index();
        self.map.keys.insert(self.key, idx);
        self.map.data.insert(idx, (1, value));
        self.map.data.get_mut(&idx).map(|(_, v)| v).unwrap()
    }
}
