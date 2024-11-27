#![forbid(unsafe_code)]

use std::{fs, io};
use std::borrow::Borrow;
use std::io::Error;
use std::mem::swap;
use std::ops::{Deref, DerefMut};
use std::path::Path;
use crate::handle::{DirHandle, FileHandle, Handle};

type Callback<'a> = dyn FnMut(&mut Handle) + 'a;

#[derive(Default)]
pub struct Walker<'a> {
    callbacks: Vec<Box<Callback<'a>>>,
}

impl<'a> Walker<'a> {
    pub fn new() -> Self {
        // TODO: your code goes here.
        Self { callbacks: vec![] }
    }

    pub fn add_callback<F>(&mut self, callback: F)
    where
        F: FnMut(&mut Handle) + 'a,
    {
        // TODO: your code goes here.
        self.callbacks.push(Box::new(callback));
    }

    pub fn walk<P: AsRef<Path>>(mut self, path: P) -> io::Result<()> {
        let mut error: Option<io::Error> = None;
        let mut callbacks: Vec<&mut Callback> = self.callbacks.iter_mut().map(|mut x| Box::as_mut(x)).collect();
        Self::traverse(path.as_ref(), &mut error, callbacks.as_mut_slice());
        match error {
            None => {
                Ok(())
            }
            Some(err) => {
                Err(err)
            }
        }
    }
    fn traverse(path: &Path, error: &mut Option<io::Error>, callbacks: &mut [&mut Callback])
    {
        let try_exist = path.try_exists();
        if try_exist.is_err() {
            *error = Some(try_exist.unwrap_err());
            return;
        }
        if !try_exist.unwrap() {
            *error = Some(io::Error::from(io::ErrorKind::NotFound));
        }
        if path.is_dir() {

            let mut index = 0;
            let mut to_swap = vec![];
            for (i, x) in callbacks.into_iter().enumerate() {
                let mut handle = Handle::Dir(DirHandle::new(path));
                let m_h = &mut handle;
                (**x)(m_h);
                let d = m_h.get_dir();
                if d.descend {
                    to_swap.push((index, i));
                    index+=1;
                    d.descend = false;
                }
            }
            for x in to_swap {
                callbacks.swap(x.0, x.1);
            }

            let children = fs::read_dir(path);
            if children.is_err() {
                *error = Some(children.unwrap_err());
                return;
            }
            let children = children.unwrap();
            for x in children {
                match x {
                    Ok(dir) => {
                        Self::traverse(dir.path().as_path(), error, &mut callbacks[..index]);
                    }
                    Err(v) => {
                        *error = Some(v);
                        continue;
                    }
                }
            }
            return;
        }
        let mut handle = Handle::File(FileHandle::new(path));
        let mut will_called = vec![];
        for x in callbacks {
            (**x)(&mut handle);
            let d = handle.get_file();
            if d.read {
                will_called.push(x);
                d.read = false;
            }
        }
        if will_called.is_empty() {
            return;
        }
        let content = fs::read(path);
        if content.is_err() {
            *error = Some(content.unwrap_err());
            return;
        }
        let content = content.unwrap();
        let mut handle = Handle::Content { file_path: path, content: &content };
        for f in will_called {
            f(&mut handle);
        }
    }


    // TODO: your code goes here.
}
