#![forbid(unsafe_code)]

pub use gc_derive::Scan;

use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    marker::PhantomData,
    ops::Deref,
    rc::{Rc, Weak},
};
////////////////////////////////////////////////////////////////////////////////

pub struct Gc<T:Scan+?Sized> {
    weak: Weak<T>,
}

impl<T: Scan + ?Sized> Clone for Gc<T> {
    fn clone(&self) -> Self {
        Self {
            weak: self.weak.clone(),
        }
    }
}

impl<T: Scan+'static> Gc<T> {
    pub fn borrow(&self) -> GcRef<'_, T> {
        // TODO: your code goes here.
        GcRef{ rc: self.weak.upgrade().unwrap(), lifetime: PhantomData::default()}
    }

    pub fn coerce_to_scan(self) -> Gc<dyn Scan> {
        Gc{ weak: self.weak }
    }
}

pub struct GcRef<'a, T: Scan> {
    rc: Rc<T>,
    lifetime: PhantomData<&'a Gc<T>>,
}

impl<'a, T: Scan> Deref for GcRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.rc
    }
}

////////////////////////////////////////////////////////////////////////////////

pub trait Scan {
    fn scan(&self, prev: &mut HashSet<usize>)->Vec<Gc<dyn Scan>>;
    fn refers_to(&self)->Vec<Gc<dyn Scan>>;
}
impl<T: Scan+'static> Scan for Gc<T> {
    fn scan(&self, prev: &mut HashSet<usize>) -> Vec<Gc<dyn Scan>> {
        let ptr = self.weak.as_ptr() as usize;
        if prev.contains(&ptr) {
            return vec![]
        }
        prev.insert(ptr);
        let mut out = self.weak.upgrade().unwrap().scan(prev);
        out.push(self.clone().coerce_to_scan());
        out
    }

    fn refers_to(&self) -> Vec<Gc<dyn Scan>> {
        let mut out = vec![];
        out.push(self.clone().coerce_to_scan());
        out
    }
}

impl<T: Scan+'static> Scan for Vec<Gc<T>> {
    fn scan(&self, prev: &mut HashSet<usize>) -> Vec<Gc<dyn Scan>> {
        let mut out = vec![];
        let iter = self.iter();
        for x in iter{
            x.scan(prev).into_iter().for_each(|x|{
                out.push(x.clone());
            });
            out.push(x.clone().coerce_to_scan());
        }
        out
    }

    fn refers_to(&self) -> Vec<Gc<dyn Scan>> {
        let mut out = vec![];
        for x in self{
            out.push(x.clone().coerce_to_scan());
        }
        out
    }
}
impl<T: Scan> Scan for RefCell<T> {
    fn scan(&self, prev: &mut HashSet<usize>) -> Vec<Gc<dyn Scan>> {
        self.borrow().scan(prev)
    }

    fn refers_to(&self) -> Vec<Gc<dyn Scan>> {
        self.borrow().refers_to()
    }
}

impl<T: Scan> Scan for Option<T> {
    fn scan(&self, prev: &mut HashSet<usize>) -> Vec<Gc<dyn Scan>> {
        match self {
            None => {vec![]}
            Some(v) => {v.scan(prev)}
        }
    }

    fn refers_to(&self) -> Vec<Gc<dyn Scan>> {
        match self {
            None => {vec![]}
            Some(v) => {v.refers_to()}
        }
    }
}
// TODO: your code goes here.

////////////////////////////////////////////////////////////////////////////////

pub struct Arena {
    pool: HashMap<usize, Rc<dyn Scan>>
}

impl Arena {
    pub fn new() -> Self {
        Self{ pool: HashMap::new() }
    }

    pub fn allocation_count(&self) -> usize {
        self.pool.len()
    }

    pub fn alloc<T: Scan + 'static>(&mut self, obj: T) -> Gc<T> {
        let rc = Rc::new(obj);
        let cloned = Rc::clone(&rc);
        self.pool.insert(Rc::as_ptr(&rc) as *const () as usize, cloned);
        Gc{ weak: Rc::downgrade(&rc) }
    }

    pub fn sweep(&mut self) {
        let mut refers_to = HashMap::new();
        // let mut seen = HashSet::new();
        for x in &mut self.pool{

            for scanned in x.1.refers_to() {
                let ptr = (scanned.weak.as_ptr() as *const() as usize);
                if refers_to.contains_key(&ptr) {
                    *refers_to.get_mut(&ptr).unwrap()+=1;
                }else {
                    refers_to.insert(ptr, 1);
                }
            }


        }
        let mut marked = HashSet::new();
        for x in &self.pool {
            let ptr = Rc::as_ptr(x.1) as *const() as usize;
            let refers = *refers_to.get(&ptr).unwrap_or(&0);
            if Rc::weak_count(x.1) > refers {
                marked.insert(ptr);
            }
        }
        let mut inner_marked = HashSet::new();
        for mark in &marked {
            let rc = self.pool.get(mark).unwrap();
            rc.scan(&mut HashSet::new()).iter().for_each(|x| {
                inner_marked.insert(x.weak.as_ptr() as *const() as usize);
            })
        }
        let mut to_drop = vec![];
        for ptr in &self.pool{
            if !marked.contains(ptr.0) && !inner_marked.contains(ptr.0) {
                to_drop.push(*ptr.0);
            }
        }
        for d in to_drop {
            self.pool.remove(&d);
        }
    }

    // TODO: your code goes here.
}
