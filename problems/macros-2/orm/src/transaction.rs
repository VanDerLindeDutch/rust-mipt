#![forbid(unsafe_code)]
use crate::{
    data::ObjectId,
    error::{Error, NotFoundError, Result},
    object::{Object, Schema},
    storage::StorageTransaction,
};
use std::{
    any::{Any, TypeId},
    cell::{Cell, Ref, RefCell, RefMut},
    collections::{hash_map::Entry, HashMap},
    marker::PhantomData,
    rc::Rc,
};

////////////////////////////////////////////////////////////////////////////////
type ObjectPool = HashMap<TypeId, HashMap<ObjectId, Rc<RefCell<( dyn Object, ObjectState)>>>>;
// TODO: your code goes here.
pub struct Transaction<'a> {
    pool: RefCell<ObjectPool>,
    storage: Box<dyn StorageTransaction>,
    // TODO: your code goes here.
}

impl<'a> Transaction<'a> {
    pub(crate) fn new(inner: Box<dyn StorageTransaction + 'a>) -> Self {
        Self { pool: RefCell::new(Default::default()), storage: inner }
    }

    fn ensure_table<T: Object>(&self) -> Result<()> {
        // TODO: your code goes here.
        unimplemented!()
    }
    fn get_pool(&self) -> RefMut<ObjectPool> {
        self.pool.borrow_mut()
    }
    pub fn create<T: Object>(&self, src_obj: T) -> Result<Tx<'_, T>> {
        if self.pool.contains_key(&src_obj.type_id()) {
            let mut map = self.get_pool();
            let mut map = map.get_mut(&src_obj.type_id()).unwrap();
            let rc = Rc::new(RefCell::new((src_obj, ObjectState::Clean)));
            map.insert(self.get_object_id::<T>()?, Rc::clone(&rc));
            return Ok(self)

        }
        let mut map = self.get_pool();
        map.insert(src_obj.type_id(), HashMap::default());
        let mut map = map.get_mut(src_obj.type_id()).unwrap();
        Ok(
            Tx::new(
                map.insert(self.get_object_id::<T>(), Rc::new(RefCell::new((src_obj, ObjectState::Clean)))).unwrap()
            )
        )
    }

    fn get_object_id<T: Object>(&self) -> Result<TypeId> {
        Err(Error::LockConflict)
    }

    pub fn get<T: Object>(&self, id: ObjectId) -> Result<Tx<'_, T>> {
        // TODO: your code goes here.
        unimplemented!()
    }

    fn try_apply(&self) -> Result<()> {
        // TODO: your code goes here.
        unimplemented!()
    }

    pub fn commit(self) -> Result<()> {
        // TODO: your code goes here.
        unimplemented!()
    }

    pub fn rollback(self) -> Result<()> {
        // TODO: your code goes here.
        unimplemented!()
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ObjectState {
    Clean,
    Modified,
    Removed,
}

#[derive(Clone)]
pub struct Tx<'a, T: Object> {
    object: &'a RefCell<(T, ObjectState)>,
    // TODO: your code goes here.
}

impl<'a, T: Object> Tx<'a, T> {
    fn new(object: &RefCell<T>) -> Self <T> {
        Self {
            object
        }
    }
    pub fn id(&self) -> ObjectId {
        // TODO: your code goes here.
        unimplemented!()
    }

    pub fn state(&self) -> ObjectState {
        // TODO: your code goes here.
        unimplemented!()
    }

    pub fn borrow(&self) -> Ref<'_, T> {
        self.object.borrow()
    }

    pub fn borrow_mut(&self) -> RefMut<'_, T> {
        self.object.borrow_mut()
    }

    pub fn delete(self) {
        // TODO: your code goes here.
        unimplemented!()
    }
}
