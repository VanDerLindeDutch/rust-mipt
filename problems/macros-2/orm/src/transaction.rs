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
use std::ops::Deref;
use std::ptr::NonNull;
use crate::storage::Row;
////////////////////////////////////////////////////////////////////////////////
#[repr(C)]
struct ObjectWithState<T: Object + ?Sized> {
    state: ObjectState,
    id: ObjectId,
    object: Box<T>,
}


impl<T: Object> ObjectWithState<T> {
    fn as_trait(self) -> ObjectWithState<dyn Object> {
        let object: Box<dyn Object> = Box::new(*self.object);
        ObjectWithState {
            state: self.state,
            id: self.id,
            object,
        }
    }
}


type ObjectPool = HashMap<TypeId, HashMap<ObjectId, RefCell<ObjectWithState<dyn Object>>>>;
// TODO: your code goes here.
pub struct Transaction<'a> {
    pool: RefCell<ObjectPool>,
    storage: Box<dyn StorageTransaction + 'a>,
}

impl<'a> Transaction<'a> {
    pub(crate) fn new(inner: Box<dyn StorageTransaction + 'a>) -> Self {
        Self { pool: RefCell::new(Default::default()), storage: inner }
    }

    pub fn get<T: Object>(&self, id: ObjectId) -> Result<Tx<'_, T>> {
        let mut map = self.pool.borrow_mut();
        let mut in_map = map.get_mut(&TypeId::of::<T>());
        if in_map.is_none() {
            let new_map: HashMap<ObjectId, RefCell<ObjectWithState<dyn Object>>> = HashMap::new();
            map.insert(TypeId::of::<T>(), new_map);
            in_map = map.get_mut(&TypeId::of::<T>());
        }
        let in_map = in_map.unwrap();

        let mut val = in_map.get(&id);
        if val.is_none() {
            let v = self.storage.select_row(id, &T::get_s())?;
            let new_row = T::from_schema(T::get_s(), v);
            in_map.insert(id, RefCell::new(ObjectWithState {
                state: ObjectState::Clean,
                id,
                object: Box::new(new_row),
            }));
            val = in_map.get(&id);
        }
        let val = val.unwrap();
        if val.borrow().state == ObjectState::Removed {
            return Err(Error::NotFound(Box::new(NotFoundError { object_id: id, type_name: val.borrow().object.get_schema().table_name })));
        }
        let val = unsafe { std::mem::transmute::<&RefCell<ObjectWithState<dyn Object>>, &RefCell<ObjectWithState<T>>>(val) };
        let r = std::ptr::from_ref(val.deref());
        Ok(Tx::new(r))
    }
    pub fn create<T: Object>(&self, src_obj: T) -> Result<Tx<'_, T>> {
        if self.pool.borrow().contains_key(&src_obj.type_id()) {
            if let Some(v) = self.pool.borrow().get(&src_obj.type_id()) {
                if v.is_empty() && !self.storage.table_exists(&src_obj.get_schema().table_name)? {
                    self.storage.create_table(&src_obj.get_schema())?;
                }
            }
            let obj_id = self.storage.insert_row(&src_obj.get_schema(), &src_obj.get_values())?;

            let mut qmap = RefMut::map(self.pool.borrow_mut(), |map| {
                map.get_mut(&src_obj.type_id()).unwrap()
            });
            let obj_with_state = ObjectWithState { state: ObjectState::Clean, object: Box::new(src_obj), id: obj_id };
            let rc = obj_with_state.as_trait();

            qmap.insert(obj_id, RefCell::new(rc));
            drop(qmap);
            return self.get(obj_id);
        }
        if !self.storage.table_exists(&src_obj.get_schema().table_name)? {
            self.storage.create_table(&src_obj.get_schema())?;
        }
        let obj_id = self.storage.insert_row(&src_obj.get_schema(), &src_obj.get_values())?;
        let mut map = self.pool.borrow_mut();
        map.insert(src_obj.type_id(), HashMap::default());
        let mut ref_map = map.get_mut(&src_obj.type_id()).unwrap();
        let obj_with_state = ObjectWithState { state: ObjectState::Clean, object: Box::new(src_obj), id: obj_id };

        let rc = RefCell::new(obj_with_state.as_trait());

        ref_map.insert(obj_id, rc);
        drop(map);
        return self.get(obj_id);
    }
    fn ensure_table<T: Object>(&self) -> Result<()> {
        // TODO: your code goes here.
        unimplemented!()
    }

    fn try_apply(&self) -> Result<()> {
        // TODO: your code goes here.
        unimplemented!()
    }

    pub fn commit(self) -> Result<()> {
        let map = self.pool.take();
        for (t, h) in map {
            for x in h {
                match x.1.borrow().state {
                    ObjectState::Clean => {}
                    ObjectState::Modified => {
                        self.storage.update_row(x.0, &x.1.borrow().object.get_schema(), &x.1.borrow().object.get_values())?;
                    }
                    ObjectState::Removed => {
                        self.storage.delete_row(x.0, &x.1.borrow().object.get_schema())?;
                    }
                }
            }
        }
        self.storage.commit()?;
        Ok(())
    }

    pub fn rollback(self) -> Result<()> {
        self.storage.rollback()?;
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ObjectState {
    Clean,
    Modified,
    Removed,
}

pub struct Tx<'a, T: Object> {
    object: *const RefCell<ObjectWithState<T>>,
    // my: My<'a, T>
    phantom_data: PhantomData<&'a T>,
    // TODO: your code goes here.
}

impl<'a, T: Object> Clone for Tx<'a, T> {
    fn clone(&self) -> Self {
        Self { object: self.object, phantom_data: PhantomData {} }
    }
}

impl<'a, T: Object> Tx<'a, T> {
    fn new<'b>(object: *const RefCell<ObjectWithState<T>>) -> Tx<'a, T> {
        Self {
            object,
            phantom_data: PhantomData {},
        }
    }
    pub fn id(&self) -> ObjectId {
        unsafe { Ref::map((*self.object).borrow(), |x| &x.id) }.clone()
    }

    pub fn state(&self) -> ObjectState {
        unsafe { Ref::map((*self.object).borrow(), |x| &x.state) }.clone()
    }

    pub fn borrow(&'a self) -> Ref<'a, T> {

        unsafe {
            if (*self.object).borrow().state == ObjectState::Removed {
                panic!("cannot borrow a removed object")
            }
            Ref::map((*self.object).borrow(), |x| x.object.as_ref())
        }
    }

    pub fn borrow_mut(&self) -> RefMut<'_, T> {
        unsafe {
            if (*self.object).borrow().state == ObjectState::Removed {
                panic!("cannot borrow a removed object")
            }
            (*self.object).borrow_mut().state = ObjectState::Modified;
            RefMut::map((*self.object).borrow_mut(), |x| x.object.as_mut())
        }
    }

    pub fn delete(self) {
        unsafe {
            if (*self.object).try_borrow_mut().is_err() {
                panic!("cannot delete a borrowed object")
            }
            (*self.object).borrow_mut().state = ObjectState::Removed;
        }
    }
}
