use crate::node::Node;
use crate::world::ref_node;
use crate::world::World;
use lilv_sys::*;
use std::rc::Rc;

pub struct ScalePoint {
    pub(crate) point: *const LilvScalePoint,
    pub(crate) world: Rc<World>,
}

impl ScalePoint {
    pub fn get_label(&self) -> Node {
        ref_node(&self.world, unsafe {
            lilv_scale_point_get_label(self.point)
        })
    }

    pub fn get_value(&self) -> Node {
        ref_node(&self.world, unsafe {
            lilv_scale_point_get_value(self.point)
        })
    }
}
