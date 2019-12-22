use crate::collection::*;
use crate::port::Port;
use crate::scale_point::ScalePoint;
use lilv_sys as lib;
use std::ptr::NonNull;

pub type ScalePoints<'a> =
    Collection<lib::LilvScalePoints, lib::LilvScalePoint, ScalePoint<'a>, &'a Port<'a>>;

impl<'a> ScalePoints<'a> {
    pub fn size(&self) -> usize {
        unsafe { lib::lilv_scale_points_size(self.inner.read().as_ptr()) as _ }
    }
}

impl<'a> CollectionTrait for ScalePoints<'a> {
    type Inner = lib::LilvScalePoints;
    type InnerTarget = lib::LilvScalePoint;
    type Target = ScalePoint<'a>;
    type Owner = &'a Port<'a>;

    unsafe fn inner(&self) -> *const Self::Inner {
        self.inner.read().as_ptr()
    }

    fn begin_fn() -> BeginFn<Self> {
        lib::lilv_scale_points_begin
    }

    fn is_end_fn() -> IsEndFn<Self> {
        lib::lilv_scale_points_is_end
    }

    fn get_fn() -> GetFn<Self> {
        lib::lilv_scale_points_get
    }

    fn next_fn() -> NextFn<Self> {
        lib::lilv_scale_points_next
    }

    fn free_fn() -> FreeFn<Self> {
        lib::lilv_scale_points_free
    }

    fn get(&self, i: *mut lib::LilvIter) -> Self::Target {
        ScalePoint::new(
            NonNull::new(unsafe { Self::get_fn()(self.inner(), i) as _ }).unwrap(),
            self.owner,
        )
    }
}
