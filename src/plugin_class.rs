use crate::node::Node;
use crate::plugin_classes::PluginClasses;
use crate::world::InnerWorld;
use lilv_sys as lib;
use parking_lot::RwLock;
use std::ptr::NonNull;
use std::sync::Arc;

unsafe impl Send for PluginClass {}
unsafe impl Sync for PluginClass {}

pub struct PluginClass {
    pub(crate) inner: RwLock<NonNull<lib::LilvPluginClass>>,
    world: Arc<InnerWorld>,
}

impl PluginClass {
    pub(crate) fn new_borrowed(ptr: NonNull<lib::LilvPluginClass>, world: Arc<InnerWorld>) -> Self {
        Self {
            inner: RwLock::new(ptr),
            world,
        }
    }

    pub fn parent_uri(&self) -> Option<Node> {
        let inner = self.inner.read().as_ptr();

        Some(Node::new_borrowed(
            NonNull::new(unsafe { lib::lilv_plugin_class_get_parent_uri(inner) as _ })?,
            self.world.clone(),
        ))
    }

    pub fn uri(&self) -> Node {
        let inner = self.inner.read().as_ptr();

        Node::new_borrowed(
            NonNull::new(unsafe { lib::lilv_plugin_class_get_uri(inner) as _ }).unwrap(),
            self.world.clone(),
        )
    }

    pub fn label(&self) -> Node {
        let inner = self.inner.read().as_ptr();

        Node::new_borrowed(
            NonNull::new(unsafe { lib::lilv_plugin_class_get_label(inner) as _ }).unwrap(),
            self.world.clone(),
        )
    }

    pub fn children(&self) -> PluginClasses {
        let inner = self.inner.read().as_ptr();

        PluginClasses::new(
            NonNull::new(unsafe { lib::lilv_plugin_class_get_children(inner) }).unwrap(),
            self.world.clone(),
        )
    }
}
