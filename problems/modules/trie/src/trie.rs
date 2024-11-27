// #![forbid(unsafe_code)]
use crate::trie_key::ToKeyIter;
use std::{borrow::Borrow, collections::HashMap, hash::Hash, ops::Index};
use std::collections::VecDeque;
use std::rc::Rc;

struct TrieNode<K: ToKeyIter, V> {
    hash_map: HashMap<K::Item, Box<TrieNode<K, V>>>,
    value: Option<V>,
    is_leaf: bool,
}
// TODO: your code goes here.

impl<K: ToKeyIter, V> TrieNode<K, V> {
    fn new() -> TrieNode<K, V> {
        Self {
            hash_map: HashMap::new(),

            value: None,
            is_leaf: false,
        }
    }
}
// TODO: your code goes here.

////////////////////////////////////////////////////////////////////////////////

pub struct Trie<K: ToKeyIter, V> {
    root: TrieNode<K, V>,
    len: usize,
}
// TODO: your code goes here.

impl<K: ToKeyIter, V> Trie<K, V>
// TODO: your code goes here.
{
    pub fn new() -> Self {
        // TODO: your code goes here.
        Self { root: TrieNode::new(), len: 0 }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn insert<Q: ?Sized + ToKeyIter<Item=K::Item>>(&mut self, key: &Q, value: V) -> Option<V>
    where
        K: Borrow<Q>,
    {
        let mut node = &mut self.root;
        for x in key.key_iter() {
            let v = node.hash_map.contains_key(&x);
            if v {
                node = node.hash_map.get_mut(&x)?;
                continue;
            }

            let v: Option<i32> = None;
            let new_node = Box::new(TrieNode::new());
            node.hash_map.insert(x.clone(), new_node);
            node = node.hash_map.get_mut(&x)?.as_mut();
        }
        if node.value.is_none() {
            self.len += 1;
        }
        node.value.replace(value)
    }

    pub fn get<Q: ?Sized + ToKeyIter<Item=K::Item>>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
    {
        let mut node = &self.root;
        for x in key.key_iter() {
            match node.hash_map.get(&x) {
                None => {
                    return None;
                }
                Some(v) => {
                    node = v;
                }
            }
        }
        node.value.as_ref()
    }

    pub fn get_mut<Q: ?Sized + ToKeyIter<Item=K::Item>>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
    {
        let mut node = &mut self.root;
        for x in key.key_iter() {
            match node.hash_map.get_mut(&x) {
                None => {
                    return None;
                }
                Some(v) => {
                    node = v;
                }
            }
        }
        node.value.as_mut()
    }

    pub fn contains<Q: ?Sized + ToKeyIter<Item=K::Item>>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
    {
        let mut node = &self.root;
        for x in key.key_iter() {
            match node.hash_map.get(&x) {
                None => {
                    return false;
                }
                Some(v) => {
                    node = v;
                }
            }
        }
        node.value.is_some()
    }

    pub fn starts_with<Q: ?Sized + ToKeyIter<Item=K::Item>>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
    {
        let mut node = &self.root;
        for x in key.key_iter() {
            match node.hash_map.get(&x) {
                None => {
                    return false;
                }
                Some(v) => {
                    node = v;
                }
            }
        }
        true
    }

    pub fn remove<Q: ?Sized + ToKeyIter<Item=K::Item>>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
    {
        let mut node = &mut self.root;
        let mut vec: VecDeque<(*mut TrieNode<K, V>, bool, K::Item)> = VecDeque::new();

        for x in key.key_iter() {
            let ptr = node as *mut TrieNode<K, V>;
            match node.hash_map.get_mut(&x) {
                None => {
                    return None;
                }
                Some(v) => {
                    let n = v.value.is_none();
                    let cur_len = v.hash_map.len();
                    vec.push_back((ptr, cur_len == 1 && n, x.clone()));
                    node = v;
                }
            }
        }
        if node.value.is_none() {
            return None;
        }
        self.len -= 1;
        if node.hash_map.len() == 0 {
            let last = vec.len() - 1;
            vec[last].1 = true;
        }
        let out = node.value.take();
        let mut iter = vec.into_iter();
        while let Some((v, drop, key)) = iter.next_back() {
            if !drop {
                break;
            }
            unsafe {
                let mut v = &mut *v;
                v.hash_map.remove(&key);
            }
        }
        out
    }
}

////////////////////////////////////////////////////////////////////////////////

impl<K: ToKeyIter + Borrow<Q>, V, Q: ?Sized + ToKeyIter<Item=K::Item>> Index<&Q> for Trie<K, V> {
    type Output = V;

    fn index(&self, index: &Q) -> &Self::Output {
        let mut node = &self.root;
        for x in index.key_iter() {
            match node.hash_map.get(&x) {
                None => {
                    panic!()
                }
                Some(v) => {
                    node = v;
                }
            }
        }

        node.value.as_ref().unwrap()
    }
}
// TODO: your code goes here.
