use crate::collection::Collection;
use crate::collection::Iter;
use crate::scale_point::ScalePoint;
use crate::world::World;
use crate::Void;
use std::rc::Rc;

#[link(name = "lilv-0")]
extern "C" {
    fn lilv_scale_points_free(scale_points: *mut Void);
    fn lilv_scale_points_size(scale_points: *const Void) -> u32;
    fn lilv_scale_points_begin(scale_points: *const Void) -> *mut Void;
    fn lilv_scale_points_get(scale_points: *const Void, i: *mut Void) -> *const Void;
    fn lilv_scale_points_next(scale_points: *const Void, i: *mut Void) -> *mut Void;
    fn lilv_scale_points_is_end(scale_points: *const Void, i: *mut Void) -> u8;
}

pub struct ScalePoints {
    pub(crate) scale_points: *const Void,
    pub(crate) owned: bool,
    pub(crate) world: Rc<World>,
}

impl AsRef<*const Void> for ScalePoints {
    fn as_ref(&self) -> &*const Void {
        &self.scale_points
    }
}

impl<'a> Collection<'a> for ScalePoints
where
    Self: 'a,
{
    type Target = ScalePoint;

    fn get(&self, i: *mut Void) -> Self::Target {
        ScalePoint {
            point: unsafe { lilv_scale_points_get(self.scale_points, i) } as *mut Void,
            world: self.world.clone(),
        }
    }
}

impl ScalePoints {
    pub fn iter<'a>(&'a self) -> Iter<'a, Self> {
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
