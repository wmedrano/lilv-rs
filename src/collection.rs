use crate::Void;
use std::ptr;

pub trait Collection<'a>: Sized + AsRef<*const Void> {
    type Target;

    unsafe fn get(&self, i: *mut Void) -> Self::Target;
}

type BeginFn = unsafe extern "C" fn(*const Void) -> *mut Void;
type IsEndFn = unsafe extern "C" fn(*const Void, *mut Void) -> bool;
type NextFn = unsafe extern "C" fn(*const Void, *mut Void) -> *mut Void;

pub struct Iter<'a, C> {
    iter: *mut Void,
    collection: &'a C,
    begin: BeginFn,
    is_end: IsEndFn,
    next: NextFn,
}

impl<'a, C> Iter<'a, C>
where
    C: Collection<'a>,
{
    pub(crate) fn new(collection: &'a C, begin: BeginFn, is_end: IsEndFn, next: NextFn) -> Self {
        Self {
            iter: ptr::null_mut(),
            collection,
            begin,
            next,
            is_end,
        }
    }
}

impl<'a, C> Iterator for Iter<'a, C>
where
    C: Collection<'a>,
{
    type Item = C::Target;

    fn next(&mut self) -> Option<Self::Item> {
        if unsafe { (self.is_end)(*self.collection.as_ref(), self.iter) } {
            return None;
        } else if self.iter.is_null() {
            self.iter = unsafe { (self.begin)(*self.collection.as_ref()) };
        } else {
            self.iter = unsafe { (self.next)(*self.collection.as_ref(), self.iter) };
        }
        Some(unsafe { self.collection.get(self.iter) })
    }
}
