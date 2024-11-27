#![forbid(unsafe_code)]

use std::env::var;
use std::fmt::{Debug, Display};
use log::log;
use crate::node::Node;

pub struct AVLTreeMap<K, V> {
    pub root: Option<Box<Node<K, V>>>,
    len: usize,
    // TODO: your code goes here.
}

impl<K: Ord + Display, V: Display> Default for AVLTreeMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Ord + Display, V: Display> AVLTreeMap<K, V> {
    pub fn new() -> Self {
        Self { root: None, len: 0 }
    }

    pub fn len(&self) -> usize {
        // TODO: your code goes here.
        self.len
    }

    pub fn is_empty(&self) -> bool {
        // TODO: your code goes here.
        self.len == 0
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let mut n = self.root.as_ref();
        loop {
            if n?.key.eq(key) {
                return Some(&n?.value);
            }
            if n?.key.gt(&key) && n?.left.is_some() {
                n = n?.left.as_ref();
                continue;
            }
            if n?.key.lt(&key) && n?.right.is_some() {
                n = n?.right.as_ref();
                continue;
            }
            return None;
        }
    }
    pub fn get_key_value<'a, 'b>(&'a self, key: &'a K) -> Option<(&'a K, &'a V)> {
        match self.get(key) {
            None => {
                None
            }
            Some(v) => {
                Some((key, v))
            }
        }
    }
    pub fn contains_key<>(&self, key: &K) -> bool {
        self.get(key).is_some()
    }
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let out: &mut Option<V> = &mut None;
        self.root = Some(Node::Insert(self.root.take(), key, value, out));
        match out.take() {
            None => {
                self.len += 1;
                None
            }
            Some(v) => {
                Some(v)
            }
        }
    }

    pub fn nth_key_value(&self, k: usize) -> Option<(&K, &V)> {
        if k >= self.len {
            return None;
        }
        let out: &mut Option<(&K, &V)> = &mut None;
        Node::inorderTraversal(&self.root, &mut 0, k, out);
        out.take()
    }
    pub fn remove(&mut self, key: &K) -> Option<V> {
        let value: &mut Option<(K, V)> = &mut None;
        self.root = Node::remove(self.root.take(), key, value);
        match value.take() {
            None => { None }
            Some(v) => {
                self.len -= 1;
                Some(v.1)
            }
        }
    }
    pub fn remove_entry(&mut self, key: &K) -> Option<(K, V)> {
        let value: &mut Option<(K, V)> = &mut None;
        self.root = Node::remove(self.root.take(), key, value);
        match value.take() {
            None => { None }
            Some(v) => {
                self.len -= 1;
                Some(v)
            }
        }
    }

    pub fn print(&self) {
        Node::print(&self.root, String::new(), false)
    }
}
