#![forbid(unsafe_code)]

use std::cell::{Ref, RefCell};
use std::collections::VecDeque;
use std::fmt::{Debug, Error, Formatter};
use std::iter::{Cloned, Peekable};
use std::rc::Rc;
use std::slice::Iter;
use std::vec::IntoIter;

pub struct LazyCycle<I>
where
    I: Iterator,
    I::Item: Clone,
{
    iter: I,
    vec: VecDeque<I::Item>,
    last: bool,
    // TODO: your code goes here.
}

impl<I: Iterator> Iterator for LazyCycle<I>
where
    I::Item: Clone,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.last {
            let out = self.vec.pop_front().unwrap();
            self.vec.push_back(out.clone());
            return Some(out);
        }
        let val = self.iter.next();
        match val {
            None => {
                self.last = true;
                if self.vec.is_empty() {
                    return None;
                }
                let out = self.vec.pop_front().unwrap();
                self.vec.push_back(out.clone());
                Some(out)
            }
            Some(v) => {
                self.vec.push_back(v.clone());
                Some(v)
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct Extract<'a, I: Iterator> {
    iter: Box<dyn Iterator<Item=I::Item> + 'a>,

    // TODO: your code goes here.
}

impl<'a, I: Iterator> Extract<'a, I> {
    fn new(vec: impl 'a + Iterator<Item=I::Item>) -> Extract<'a, I> {
        Self { iter: Box::new(vec) }
    }
}

impl<I: Iterator> Iterator for Extract<'_, I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<I::Item> {
        self.iter.next()
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct Tee<I>
where
    I: Iterator,
    I::Item: Clone,
{
    shared_iter: Rc<RefCell<VecDeque<I::Item>>>,
    own_vec: Rc<RefCell<VecDeque<I::Item>>>,
    local_iter: Rc<RefCell<I>>,
    is_empty: Rc<RefCell<bool>>,
    // TODO: your code goes here.
}

impl<I> Tee<I>
where
    I: Iterator,
    I::Item: Clone,
{
    fn new(shared_iter: Rc<RefCell<VecDeque<I::Item>>>, own_vec: Rc<RefCell<VecDeque<I::Item>>>,
           local_iter: Rc<RefCell<I>>, is_empty: Rc<RefCell<bool>>) -> Self {
        Self { shared_iter, local_iter, own_vec, is_empty }
    }
}
impl<I: Iterator> Iterator for Tee<I>
where
    I::Item: Clone,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {

        if !self.own_vec.borrow().is_empty() {
            let mut vec = self.own_vec.borrow_mut();
            return vec.pop_front();
        }
        if *self.is_empty.borrow() {
            return None;
        }
        match self.local_iter.borrow_mut().next() {
            None => {
                *self.is_empty.borrow_mut() = true;
                None
            }
            Some(v) => {
                self.shared_iter.borrow_mut().push_back(v.clone());
                Some(v)
            }
        }
    }
}
////////////////////////////////////////////////////////////////////////////////

pub struct GroupBy<I, F, V>
where
    I: Iterator,
    F: FnMut(&I::Item) -> V,
    V: Eq,
{
    iter: Peekable<I>,
    f: Box<F>,
    is_empty: bool,
}

impl<I, F, V> Iterator for GroupBy<I, F, V>
where
    I: Iterator,
    F: FnMut(&I::Item) -> V,
    V: Eq,
{
    type Item = (V, Vec<I::Item>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_empty {
            return None;
        }
        let val = self.iter.next();
        if val.is_none() {
            self.is_empty = true;
            return None;
        }
        let val = val.unwrap();
        let fnVal = (self.f)(&val);
        let mut out = vec![val];
        loop {
            let x = self.iter.peek();
            if x.is_none() {
                self.is_empty = true;
                return Some((fnVal, out));
            }
            if fnVal.eq(&(self.f)(x.unwrap()?)) {
                let val = self.iter.next().unwrap();
                out.push(val);
                continue;
            }
            break;
        }
        Some((fnVal, out))
    }
}

////////////////////////////////////////////////////////////////////////////////

pub trait ExtendedIterator: Iterator {
    fn lazy_cycle(self) -> LazyCycle<Self>
    where
        Self: Sized,
        Self::Item: Clone,
    {
        LazyCycle { iter: self, vec: VecDeque::new(), last: false }
    }

    fn extract<'a>(mut self, index: usize) -> (Option<Self::Item>, Extract<'a, Self>)
    where
        Self: Sized,
        Self: 'a,
    {
        let mut out: Vec<Self::Item> = Vec::<Self::Item>::new();
        let mut i = 0;
        let mut o = Option::None;
        for x in &mut self {
            if i == index {
                i += 1;
                o = Some(x);
                break;
            }
            i += 1;
            out.push(x);
        }

        (o, Extract::new(out.into_iter().chain(self)))
    }


    fn tee(self) -> (Tee<Self>, Tee<Self>)
    where
        Self: Sized,
        Self::Item: Clone,
    {
        let iter1 = Rc::new(RefCell::new(self));
        let iter2 = Rc::clone(&iter1);
        let vec1 = Rc::new(RefCell::new(VecDeque::new()));
        let vec2 = Rc::new(RefCell::new(VecDeque::new()));
        let empty = Rc::new(RefCell::new(false));
        (Tee { is_empty: Rc::clone(&empty), shared_iter: Rc::clone(&vec2), local_iter: iter1, own_vec: Rc::clone(&vec1) }, Tee { is_empty: Rc::clone(&empty), shared_iter: vec1, local_iter: iter2, own_vec: vec2 })
    }

    fn group_by<F, V>(self, func: F) -> GroupBy<Self, F, V>
    where
        Self: Sized,
        F: FnMut(&Self::Item) -> V,
        V: Eq,
    {
        GroupBy { f: Box::new(func), iter: self.peekable(), is_empty: false }
    }
}

impl<T> ExtendedIterator for T
where
    T: Iterator,
{}

