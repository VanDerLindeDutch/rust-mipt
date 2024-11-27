#![forbid(unsafe_code)]

use std::{cell::RefCell, collections::VecDeque, fmt::Debug, rc::Rc};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use thiserror::Error;
use crate::CloseMessage::Value;
////////////////////////////////////////////////////////////////////////////////

// TODO: your code goes here.

////////////////////////////////////////////////////////////////////////////////

#[derive(Error, Debug)]
#[error("channel is closed")]
pub struct SendError<T> {
    pub value: T,
}

enum CloseMessage<T> {
    Closed,
    Value(T),
}

pub struct RcSender<T> (
    Rc<RefCell<Sender<T>>>
);

pub struct Sender<T> {
    q: Rc<RefCell<VecDeque<CloseMessage<T>>>>,
    senders_count: Rc<RefCell<isize>>,
    closed: Rc<RefCell<bool>>,
}

impl<T> Sender<T> {
    pub fn send(&self, value: T) -> Result<(), SendError<T>> {
        if *self.closed.borrow() == true {
            return Err(SendError { value });
        };
        self.q.borrow_mut().push_back(Value(value));
        Ok(())
    }

    pub fn is_closed(&self) -> bool {
        *self.closed.borrow()
    }

    pub fn same_channel(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.q, &other.q)
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        *self.senders_count.borrow_mut() += 1;
        Self { q: Rc::clone(&self.q), senders_count: Rc::clone(&self.senders_count), closed: Rc::clone(&self.closed) }
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        *self.senders_count.borrow_mut() -= 1;
    }
}

impl<T> PartialEq for RcSender<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}


impl<T> Eq for RcSender<T> {}

impl<T> Hash for RcSender<T> {
    fn hash<H>(&self, hasher: &mut H)
    where
        H: Hasher,
    {
        hasher.write_usize(Rc::as_ptr(&self.0) as usize);
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Error, Debug)]
pub enum ReceiveError {
    #[error("channel is empty")]
    Empty,
    #[error("channel is closed")]
    Closed,
}

pub struct Receiver<T> {
    q: Rc<RefCell<VecDeque<CloseMessage<T>>>>,
    senders_count: Rc<RefCell<isize>>,
    closed: Rc<RefCell<bool>>,
}

impl<T> Receiver<T> {
    pub fn recv(&mut self) -> Result<T, ReceiveError> {
        if *self.senders_count.borrow() == 0 {
            return Err(ReceiveError::Closed);
        }
        let mut q = self.q.borrow_mut();
        if q.is_empty() {
            return Err(ReceiveError::Empty);
        }
        match q.pop_front().unwrap() {
            CloseMessage::Closed => {
                *self.closed.borrow_mut() = true;
                Err(ReceiveError::Closed)
            }
            CloseMessage::Value(V) => {
                Ok(V)
            }
        }
    }

    pub fn close(&mut self) {
        self.q.borrow_mut().push_back(CloseMessage::Closed);
        *self.closed.borrow_mut() = true;
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        *self.closed.borrow_mut() = true;
    }
}

////////////////////////////////////////////////////////////////////////////////

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let q = Rc::new(RefCell::new(VecDeque::<CloseMessage<T>>::new()));
    let sendersCount = Rc::new(RefCell::new(1));
    let closed = Rc::new(RefCell::new(false));
    let mut out = (Sender { q: Rc::clone(&q), senders_count: Rc::clone(&sendersCount), closed: Rc::clone(&closed) }, Receiver { q, senders_count: sendersCount, closed: Rc::clone(&closed) });
    out
}
