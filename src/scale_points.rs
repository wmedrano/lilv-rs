use crate::scale_point::ScalePoint;
use crate::world::World;
use crate::Void;
use lilv_sys::*;
use std::rc::Rc;

pub struct ScalePoints {
    pub(crate) scale_points: *const LilvScalePoints,
    pub(crate) owned: bool,
    pub(crate) world: Rc<World>,
}

impl ScalePoints {
    pub fn iter(&self) -> ScalePointIter<'_> {
        ScalePointIter {
            scale_points: self,
            iter: unsafe { lilv_scale_points_begin(self.scale_points) },
        }
    }

    pub fn len(&self) -> usize {
        unsafe { lilv_scale_points_size(self.scale_points) as usize }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Drop for ScalePoints {
    fn drop(&mut self) {
        if self.owned {
            unsafe { lilv_scale_points_free(self.scale_points as *mut Void) }
        }
    }
}

pub struct ScalePointIter<'a> {
    scale_points: &'a ScalePoints,
    iter: *mut LilvIter,
}

impl<'a> Iterator for ScalePointIter<'a> {
    type Item = ScalePoint;

    fn next(&mut self) -> Option<ScalePoint> {
        let ptr = unsafe { lilv_scale_points_get(self.scale_points.scale_points, self.iter) };
        if ptr.is_null() {
            None
        } else {
            self.iter =
                unsafe { lilv_scale_points_next(self.scale_points.scale_points, self.iter) };
            Some(ScalePoint {
                point: ptr,
                world: self.scale_points.world.clone(),
            })
        }
    }
}
