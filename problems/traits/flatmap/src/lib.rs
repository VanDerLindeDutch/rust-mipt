#![forbid(unsafe_code)]

use std::{borrow::Borrow, iter::FromIterator, ops::Index};
use std::collections::HashMap;
use std::fmt::{Display, Formatter, Write};
use std::hash::Hash;
use std::ops::{AddAssign, BitOr, Deref, Shr};
use std::task::Wake;
////////////////////////////////////////////////////////////////////////////////

#[derive(Default, Debug, PartialEq, Eq)]
pub struct FlatMap<K, V>(Vec<(K, V)>);

impl<K: Ord, V> FlatMap<K, V> {
    pub fn new() -> Self {
        // TODO: your code goes here.
        FlatMap(Vec::new())
    }

    pub fn len(&self) -> usize {
        // TODO: your code goes here.
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        // TODO: your code goes here.
        self.0.is_empty()
    }

    pub fn capacity(&self) -> usize {
        // TODO: your code goes here.
        self.0.capacity()
    }

    pub fn as_slice(&self) -> &[(K, V)] {
        // TODO: your code goes here.
        self.0.as_slice()
    }
    pub fn test_bor(&self)->&Self {
        self
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        // TODO: your code goes here.
        let res = self.0.binary_search_by(|x| x.0.cmp(&key));
        match res {
            Ok(ind) => {
                self.0.push((key, value));
                let out = self.0.swap_remove(ind);
                Some(out.1)
            }
            Err(ind) => {
                self.0.insert(ind, (key, value));
                None
            }
        }
    }

    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord + Eq,
    {
        let res = self.0.binary_search_by(|x| x.0.borrow().cmp(key));
        match res {
            Ok(ind) => {
                let out = self.0.get(ind).unwrap();
                Some(&out.1)
            }
            Err(_) => {
                None
            }
        }
    }

    pub fn remove<Q: ?Sized>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Ord + Eq,
    {
        let res = self.0.binary_search_by(|x| x.0.borrow().cmp(key));
        match res {
            Ok(ind) => {
                Some(self.0.remove(ind).1)
            }
            Err(ind) => {
                None
            }
        }
    }

    pub fn remove_entry<Q: ?Sized>(&mut self, key: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q>,
        Q: Ord + Eq,
    {
        let res = self.0.binary_search_by(|x| x.0.borrow().cmp(key));
        match res {
            Ok(ind) => {
                Some(self.0.remove(ind))
            }
            Err(ind) => {
                None
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

impl<Q: ?Sized + Ord, K: Ord + Borrow<Q>, V> Index<&Q> for FlatMap<K, V> {
    type Output = V;

    fn index(&self, index: &Q) -> &Self::Output {
        let map = HashMap::<i32, i32>::new();

        let res = self.0.binary_search_by(|x| x.0.borrow().cmp(index));
        match res {
            Ok(ind) => {
                let out = self.0.get(ind).unwrap();
                &out.1
            }
            Err(_) => {
                panic!()
            }
        }
    }
}

impl<K: Ord,V> Extend<(K, V)> for FlatMap<K, V> {
    fn extend<T: IntoIterator<Item=(K, V)>>(&mut self, iter: T) {
        for x in iter {
            self.insert(x.0, x.1);
        }
    }
}


impl<K ,V> AsRef<f32> for FlatMap<K, V> {

    fn as_ref(&self) -> &f32 {
        todo!()
    }
}
impl<K: Ord, V> From<Vec<(K, V)>> for FlatMap<K, V> {
    fn from(value: Vec<(K, V)>) -> Self {
        let mut map = Self(Vec::new());
        for x in value {
            map.insert(x.0, x.1);
        }
        map
    }
}

impl<K:Ord, V> From<FlatMap<K, V>> for Vec<(K, V)> {
    fn from(value: FlatMap<K, V>) -> Self {
        value.0
    }
}

impl<K: Ord, V> FromIterator<(K, V)> for FlatMap<K, V> {
    fn from_iter<T: IntoIterator<Item=(K, V)>>(iter: T) -> Self {
        let mut map = Self(Vec::new());
        for x in iter.into_iter() {
            map.insert(x.0, x.1);
        }
        map
    }
}

impl<K,V> IntoIterator for FlatMap<K,V> {
    type Item = (K,V);
    type IntoIter = std::vec::IntoIter<(K,V)>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

