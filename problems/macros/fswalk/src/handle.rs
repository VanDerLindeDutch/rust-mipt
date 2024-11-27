#![forbid(unsafe_code)]

use std::path::Path;
use crate::Handle::Content;

pub enum Handle<'a> {
    Dir(DirHandle<'a>),
    File(FileHandle<'a>),
    Content {
        file_path: &'a Path,
        content: &'a [u8],
    },
}

impl<'a> Handle<'a>{

    pub fn get_dir<'b>(&'a mut self) -> &'b mut DirHandle<'a> {
        match self {
            Handle::Dir(v) => v,
            _ => panic!()
        }
    }

    pub fn get_file<'b>(&'b mut self) -> &'b mut FileHandle<'a> {
        match self {
            Handle::File(f) => f,
            _ => unimplemented!()
        }
    }
}

pub struct DirHandle<'a> {
    path: &'a Path,
    pub descend: bool,
    // TODO: your code goes here.
}

impl<'a> DirHandle<'a> {
    pub fn new(path: &'a Path) -> DirHandle {
        Self { path, descend: false }
    }
    pub fn descend(&mut self) {
        self.descend = true
    }

    pub fn path(&self) -> &Path {
        // TODO: your code goes here.
        self.path
    }
}

pub struct FileHandle<'a> {
    path: &'a Path,
    pub read: bool,
}

impl<'a> FileHandle<'a> {
    pub fn new(path: &'a Path) -> Self {
        Self { path, read: false }
    }
    pub fn read(&mut self) {
        self.read = true;
        // TODO: your code goes here.

    }

    pub fn path(&self) -> &Path {
        // TODO: your code goes here.
        self.path
    }
}
