use crate::collection::Collection;
use crate::collection::Iter;
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

impl AsRef<*const Void> for ScalePoints {
    fn as_ref(&self) -> &*const Void {
        let ptr: &*const LilvScalePoints = &self.scale_points;
        unsafe { std::mem::transmute(ptr) }
    }
}

impl<'a> Collection<'a> for ScalePoints
where
    Self: 'a,
{
    type Target = ScalePoint;

    unsafe fn get(&self, i: *mut Void) -> Self::Target {
        ScalePoint {
            point: lilv_scale_points_get(self.scale_points, i),
            world: self.world.clone(),
        }
    }
}

impl ScalePoints {
    pub fn iter(&self) -> Iter<'_, Self> {
        Iter::new(
            self,
            lilv_scale_points_begin,
            lilv_scale_points_is_end,
            lilv_scale_points_next,
        )
    }

    pub fn size(&self) -> usize {
        unsafe { lilv_scale_points_size(self.scale_points) as usize }
    }
}

impl Drop for ScalePoints {
    fn drop(&mut self) {
        if self.owned {
            unsafe { lilv_scale_points_free(self.scale_points as *mut Void) }
        }
    }
}
