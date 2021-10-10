use crate::node::Node;
use crate::port::Port;
use lilv_sys as lib;
use std::ptr::NonNull;

unsafe impl<'a> Send for ScalePoint<'a> {}
unsafe impl<'a> Sync for ScalePoint<'a> {}

pub struct ScalePoint<'a> {
    pub(crate) inner: NonNull<lib::LilvScalePoint>,
    pub(crate) port: &'a Port<'a>,
}

impl<'a> ScalePoint<'a> {
    pub fn label(&self) -> Node {
        let inner = self.inner.as_ptr();

        Node::new_borrowed(
            NonNull::new(unsafe { lib::lilv_scale_point_get_label(inner) as _ }).unwrap(),
            self.port.plugin.world.clone(),
        )
    }

    pub fn value(&self) -> Node {
        let inner = self.inner.as_ptr();

        Node::new_borrowed(
            NonNull::new(unsafe { lib::lilv_scale_point_get_value(inner) as _ }).unwrap(),
            self.port.plugin.world.clone(),
        )
    }
}
