#![forbid(unsafe_code)]

use std::ops::Deref;
use std::os::unix::raw::nlink_t;
use std::rc::Rc;

pub struct PRef<T> {
    value: T,

    // TODO: your code goes here.
}

impl<T> std::ops::Deref for PRef<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // TODO: your code goes here.
        &self.value
    }
}

////////////////////////////////////////////////////////////////////////////////


pub struct Node<T> {
    prev: Option<Rc<Node<T>>>,
    value: Rc<T>,
}
pub struct PStack<T> {
    len: usize,
    head: Option<Rc<Node<T>>>,
    // TODO: your code goes here.
}

pub struct PIter<T> {
    head: Option<Rc<Node<T>>>,
}

impl<T> Iterator for PIter<T> {
    type Item = Rc<T>;

    fn next(&mut self) -> Option<Self::Item> {
        match &self.head {
            None => None,
            Some(v) => {
                let h = &self.head.clone();
                if self.head.as_ref()?.prev.is_some() {
                    self.head = Some(Rc::clone(self.head.as_ref()?.prev.as_ref()?));
                }else {
                    self.head = None
                }
                Some(Rc::clone(&h.as_ref()?.value))
            }
        }
    }
}

impl<T> Default for PStack<T> {
    fn default() -> Self {
        // TODO: your code goes here.
        Self { head: None, len: 0 }
    }
}

impl<T> Clone for PStack<T> {
    fn clone(&self) -> Self {
        // TODO: your code goes here.
        if self.head.is_none() {
            return Self { len: 0, head: None };
        }
        Self { len: self.len, head: Some(Rc::clone(self.head.as_ref().unwrap())) }
    }
}

impl<T> PStack<T> {
    pub fn new() -> Self {
        // TODO: your code goes here.
        Self { head: None, len: 0 }
    }

    pub fn push(&self, value: T) -> Self {
        if self.head.is_none() {
            return Self { len: 1, head: Some(Rc::new(Node { value: Rc::new(value), prev: None })) };
        }
        let q = self.head.as_ref().unwrap();
        Self { head: Some(Rc::new(Node { value: Rc::new(value), prev: Some(Rc::clone(q)) })), len: self.len + 1 }
    }

    pub fn pop(&self) -> Option<(Rc<T>, Self)> {
        // TODO: your code goes here.
        match self.head.as_ref() {
            None => { None }
            Some(h) => {
                let prev = h.prev.as_ref();
                if prev.is_none() {
                    return Some(
                        (Rc::clone(&h.value),
                         Self {
                             len: self.len - 1,
                             head: None,
                         }));
                }
                Some(
                    (Rc::clone(&h.value),
                     Self {
                         len: self.len - 1,
                         head: Some(Rc::clone(prev.unwrap())),
                     }))
            }
        }
    }

    pub fn len(&self) -> usize {
        // TODO: your code goes here.
        self.len
    }

    pub fn is_empty(&self) -> bool {
        // TODO: your code goes here.
        self.len == 0
    }

    pub fn iter(&self) -> impl Iterator<Item=Rc<T>> {
        // TODO: your code goes here.
        if self.head.is_none() {
            return PIter{head: None};
        }
        PIter { head: Some(Rc::clone(self.head.as_ref().unwrap())) }
    }
}

