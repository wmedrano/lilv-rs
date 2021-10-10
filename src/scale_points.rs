use crate::port::Port;
use crate::scale_point::ScalePoint;
use lilv_sys as lib;
use std::ptr::NonNull;

pub struct ScalePoints<'a> {
    pub(crate) inner: NonNull<lib::LilvScalePoints>,
    pub(crate) port: &'a Port<'a>,
}

impl<'a> ScalePoints<'a> {
    pub fn size(&self) -> usize {
        let size: u32 = unsafe { lib::lilv_scale_points_size(self.inner.as_ptr() as _) };
        size as usize
    }

    pub fn iter(&self) -> ScalePointsIter<'_> {
        ScalePointsIter {
            inner: self,
            iter: unsafe { lib::lilv_scale_points_begin(self.inner.as_ptr() as _) },
        }
    }
}

pub struct ScalePointsIter<'a> {
    inner: &'a ScalePoints<'a>,
    iter: *mut lib::LilvIter,
}

impl<'a> Iterator for ScalePointsIter<'a> {
    type Item = ScalePoint<'a>;

    fn next(&mut self) -> Option<ScalePoint<'a>> {
        let next_ptr =
            unsafe { lib::lilv_scale_points_get(self.inner.inner.as_ptr() as _, self.iter as _) }
                as *mut _;
        let next = Some(ScalePoint {
            inner: NonNull::new(next_ptr)?,
            port: self.inner.port,
        });
        self.iter =
            unsafe { lib::lilv_scale_points_next(self.inner.inner.as_ptr() as _, self.iter) };
        next
    }
}
