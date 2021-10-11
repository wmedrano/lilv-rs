use crate::node::Node;
use crate::world::Life;
use lilv_sys as lib;
use parking_lot::RwLock;
use std::ptr::NonNull;
use std::sync::Arc;

unsafe impl Send for PluginClass {}
unsafe impl Sync for PluginClass {}

pub struct PluginClass {
    pub(crate) inner: RwLock<NonNull<lib::LilvPluginClass>>,
    world: Arc<Life>,
}

impl PluginClass {
    pub(crate) fn new_borrowed(ptr: NonNull<lib::LilvPluginClass>, world: Arc<Life>) -> Self {
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

    pub fn uri(&self) -> Option<Node> {
        let inner = self.inner.read().as_ptr();

        Node::new_borrowed(
            NonNull::new(unsafe { lib::lilv_plugin_class_get_uri(inner) as _ })?,
            self.world.clone(),
        )
        .into()
    }

    /// # Panics
    /// Panics if the label could not be obtained.
    pub fn label(&self) -> Node {
        let inner = self.inner.read().as_ptr();

        Node::new_borrowed(
            NonNull::new(unsafe { lib::lilv_plugin_class_get_label(inner) as _ }).unwrap(),
            self.world.clone(),
        )
    }

    pub fn children(&self) -> Option<PluginClasses> {
        let inner = self.inner.read().as_ptr();
        PluginClasses {
            inner: NonNull::new(unsafe { lib::lilv_plugin_class_get_children(inner) })?,
            owner: self.world.clone(),
        }
        .into()
    }
}

pub struct PluginClasses {
    pub(crate) inner: NonNull<lib::LilvPluginClasses>,
    pub(crate) owner: Arc<Life>,
}

impl PluginClasses {
    #[must_use]
    pub fn iter(&self) -> PluginClassesIter {
        PluginClassesIter {
            classes: self.inner.as_ptr(),
            iter: unsafe { lib::lilv_plugin_classes_begin(self.inner.as_ptr()) },
            owner: self.owner.clone(),
        }
    }

    #[must_use]
    pub fn size(&self) -> usize {
        unsafe { lib::lilv_plugin_classes_size(self.inner.as_ptr()) as _ }
    }

    pub fn get_by_uri(&self, uri: &Node) -> Option<PluginClass> {
        let inner = self.inner.as_ptr();
        let uri = uri.inner.read().as_ptr();

        Some(PluginClass::new_borrowed(
            NonNull::new(unsafe { lib::lilv_plugin_classes_get_by_uri(inner, uri) as _ })?,
            self.owner.clone(),
        ))
    }
}

pub struct PluginClassesIter {
    classes: *mut lib::LilvPluginClasses,
    iter: *mut lib::LilvIter,
    owner: Arc<Life>,
}

impl Iterator for PluginClassesIter {
    type Item = PluginClass;

    fn next(&mut self) -> Option<PluginClass> {
        let ptr = unsafe { lib::lilv_plugin_classes_get(self.classes, self.iter) };
        if ptr.is_null() {
            None
        } else {
            self.iter = unsafe { lib::lilv_plugin_classes_next(self.classes, self.iter) };
            Some(PluginClass::new_borrowed(
                NonNull::new(ptr as _)?,
                self.owner.clone(),
            ))
        }
    }
}
