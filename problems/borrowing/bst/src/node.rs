#![forbid(unsafe_code)]

use std::cmp::Ordering;
use std::fmt::{Debug, Display};
use std::mem::swap;
use std::ops::Deref;

pub struct Node<K, V> {
    pub left: Option<Box<Node<K, V>>>,
    pub right: Option<Box<Node<K, V>>>,
    pub value: V,
    pub key: K,
    height: i32,
    // TODO: your code goes here.
}

impl<K: Ord + Display, V> Node<K, V> {
    pub fn new(k: K, v: V) -> Box<Self> {
        Box::new(Self {
            left: None,
            right: None,
            value: v,
            key: k,
            height: 1,
        })
    }

    pub fn rotateRight(mut node: Box<Node<K, V>>) -> Box<Node<K, V>> {
        let mut leftNode = node.left.unwrap();
        node.left = leftNode.right;
        leftNode.right = Some(node);
        Node::fixHeight(&mut leftNode.right.as_mut().unwrap());
        Node::fixHeight(&mut leftNode);
        leftNode
    }

    pub fn rotateLeft(mut node: Box<Node<K, V>>) -> Box<Node<K, V>> {
        let mut rightNode = node.right.unwrap();
        node.right = rightNode.left;
        rightNode.left = Some(node);
        Node::fixHeight(&mut rightNode.left.as_mut().unwrap());
        Node::fixHeight(&mut rightNode);
        rightNode
    }


    pub fn fixHeight(node: &mut Box<Node<K, V>>) {
        let (hl, hr) = (Node::getHeight(&node.left), Node::getHeight(&node.right));
        let mut val = hr;
        if hl > hr {
            val = hl;
        }
        node.height = val + 1;
    }

    pub fn balance(mut node: Box<Node<K, V>>) -> Box<Node<K, V>> {
        Node::fixHeight(&mut node);
        if Node::bfactor(&node) == 2 {
            if Node::bfactor(node.right.as_ref().unwrap()) < 0 {
                let r = node.right;
                node.right = Some(Node::rotateRight(r.unwrap()))
            }
            return Node::rotateLeft(node);
        }
        if Node::bfactor(&node) == -2 {
            if Node::bfactor(node.left.as_ref().unwrap()) > 0 {
                let r = node.left;
                node.left = Some(Node::rotateLeft(r.unwrap()))
            }
            return Node::rotateRight(node);
        }
        node
    }

    pub fn Insert(mut node: Option<Box<Node<K, V>>>, key: K, value: V, out: &mut Option<V>) -> Box<Node<K, V>> {
        if node.is_none() {
            return Node::new(key, value);
        }
        let mut node = node.unwrap();
        if node.key.eq(&key) {
            let v = node.value;
            *out = Some(v);
            node.value = value;
            return node;
        }
        if node.key.cmp(&key).is_ge() {
            node.left = Some(Node::Insert(node.left, key, value, out));
        } else {
            node.right = Some(Node::Insert(node.right, key, value, out));
        }
        Node::balance(node)
    }

    pub fn bfactor(node: &Box<Node<K, V>>) -> i32 {
        Node::getHeight(&node.right) - Node::getHeight(&node.left)
    }

    pub fn find(node: &Box<Node<K, V>>) {}


    pub fn getHeight(node: &Option<Box<Node<K, V>>>) -> i32 {
        match node {
            None => { 0 }
            Some(v) => { v.height }
        }
    }

    pub fn remove(mut node: Option<Box<Node<K, V>>>, key: &K, value: &mut Option<(K, V)>) -> Option<Box<Node<K, V>>> {
        if node.is_none() {
            return node;
        }
        let mut node = node?;
        match node.key.cmp(key) {
            Ordering::Less => {
                node.right = Node::remove(node.right, key, value);
            }
            Ordering::Equal => {
                let q = node.left;
                let mut r = node.right;
                *value = Some((node.key, node.value));
                if r.as_ref().is_none() {
                    return q;
                }
                let mut out = Node::removeMin(r);
                let mut minNode = out.1.unwrap();
                let rmd = out.0;
                minNode.right = rmd;
                minNode.left = q;
                return Some(Node::balance(minNode));
            }
            Ordering::Greater => {
                node.left = Node::remove(node.left, key, value);
            }
        }
        Some(Node::balance(node))
    }

    fn removeMin(mut node: Option<Box<Node<K, V>>>) -> (Option<Box<Node<K, V>>>, Option<Box<Node<K, V>>>) {
        let mut node = node.unwrap();
        if node.left.is_none() {
            let x = node.right.take();
            return (x, Some(node));
        }
        let out = Self::removeMin(node.left.take());
        node.left = out.0;
        (Some(Self::balance(node)), out.1)
    }

    pub fn print(node: &Option<Box<Node<K, V>>>, indent: String, isLeft: bool) {
        if node.is_none() {
            return;
        }
        let node = node.as_ref().unwrap();
        println!("{}|___{}\n", indent, node.key);
        let newStr = if isLeft {
            indent + "|    "
        } else {
            indent + "   "
        };
        Self::print(&node.left, newStr.clone(), true);
        Self::print(&node.right, newStr, false)
    }

    pub fn inorderTraversal<'a>(node: &'a Option<Box<Node<K, V>>>, i: &mut usize, k: usize, out: &mut Option<(&'a K, &'a V)>) {
        if node.is_none() {
            return;
        }

        Self::inorderTraversal(&node.as_ref().unwrap().left, i, k, out);

        if *i == k {
            let node = node.as_ref().unwrap();
            *out = Some((&node.key, &node.value));
            *i += 1;
            return;
        }
        *i += 1;

        Self::inorderTraversal(&node.as_ref().unwrap().right, i, k, out);
    }
    // TODO: your code goes here.
}


