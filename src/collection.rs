use crate::world::InnerWorld;
use lilv_sys as lib;
use parking_lot::RwLock;
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::sync::Arc;

pub struct Collection<I, It, T, O = Arc<InnerWorld>>
where
    Self: CollectionTrait<Inner = I, InnerTarget = It, Target = T, Owner = O>,
{
    pub(crate) inner: RwLock<NonNull<I>>,
    borrowed: bool,
    pub(crate) owner: O,
    _phantom: PhantomData<(It, T)>,
}

impl<'a, I, It, T, O> Collection<I, It, T, O>
where
    Self: CollectionTrait<Inner = I, InnerTarget = It, Target = T, Owner = O>,
{
    pub(crate) fn new(ptr: NonNull<I>, owner: O) -> Self {
        Self {
            inner: RwLock::new(ptr),
            borrowed: false,
            owner,
            _phantom: PhantomData,
        }
    }

    pub fn iter(&'a self) -> Iter<'a, Self> {
        Iter::new(self)
    }
}

impl<'a, I, It, T, O> Drop for Collection<I, It, T, O>
where
    Self: CollectionTrait<Inner = I, InnerTarget = It, Target = T, Owner = O>,
{
    fn drop(&mut self) {
        if !self.borrowed {
            unsafe { Self::free_fn()(self.inner.write().as_ptr()) }
        }
    }
}

#[doc(hidden)]
pub trait CollectionTrait: Sized {
    type Inner;
    type InnerTarget;
    type Target;
    type Owner;

    unsafe fn inner(&self) -> *const Self::Inner;
    fn begin_fn() -> BeginFn<Self>;
    fn is_end_fn() -> IsEndFn<Self>;
    fn get_fn() -> GetFn<Self>;
    fn next_fn() -> NextFn<Self>;
    fn free_fn() -> FreeFn<Self>;
    fn get(&self, i: *mut lib::LilvIter) -> Self::Target;
}

pub(crate) type BeginFn<C> =
    unsafe extern "C" fn(*const <C as CollectionTrait>::Inner) -> *mut lib::LilvIter;

pub(crate) type IsEndFn<C> =
    unsafe extern "C" fn(*const <C as CollectionTrait>::Inner, *mut lib::LilvIter) -> bool;

pub(crate) type GetFn<C> = unsafe extern "C" fn(
    *const <C as CollectionTrait>::Inner,
    *mut lib::LilvIter,
) -> *const <C as CollectionTrait>::InnerTarget;

pub(crate) type NextFn<C> = unsafe extern "C" fn(
    *const <C as CollectionTrait>::Inner,
    *mut lib::LilvIter,
) -> *mut lib::LilvIter;

pub(crate) type FreeFn<C> = unsafe extern "C" fn(*mut <C as CollectionTrait>::Inner);

pub struct Iter<'a, C>
where
    C: CollectionTrait,
{
    iter: *mut lib::LilvIter,
    collection: &'a C,
}

impl<'a, C> Iter<'a, C>
where
    C: CollectionTrait,
{
    pub(crate) fn new(collection: &'a C) -> Self {
        Self {
            iter: unsafe { C::begin_fn()(collection.inner()) },
            collection,
        }
    }
}

impl<'a, C> Iterator for Iter<'a, C>
where
    C: CollectionTrait,
{
    type Item = C::Target;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if C::is_end_fn()(self.collection.inner(), self.iter) {
                None
            } else {
                let item = self.collection.get(self.iter);
                self.iter = C::next_fn()(self.collection.inner(), self.iter);
                Some(item)
            }
        }
    }
}
