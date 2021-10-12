use crate::node::Node;
use crate::world::Life;
use lilv_sys as lib;
use std::ptr::NonNull;
use std::sync::Arc;

unsafe impl Send for PluginClass {}
unsafe impl Sync for PluginClass {}

pub struct PluginClass {
    pub(crate) inner: NonNull<lib::LilvPluginClass>,
    life: Arc<Life>,
}

impl PluginClass {
    pub(crate) fn new_borrowed(ptr: NonNull<lib::LilvPluginClass>, world: Arc<Life>) -> Self {
        Self {
            inner: ptr,
            life: world,
        }
    }

    #[must_use]
    pub fn parent_uri(&self) -> Option<Node> {
        let _life = self.life.inner.read();
        let inner = self.inner.as_ptr();

        Some(Node::new_borrowed(
            NonNull::new(unsafe { lib::lilv_plugin_class_get_parent_uri(inner) as _ })?,
            self.life.clone(),
        ))
    }

    #[must_use]
    pub fn uri(&self) -> Option<Node> {
        let _life = self.life.inner.read();
        let inner = self.inner.as_ptr();

        Node::new_borrowed(
            NonNull::new(unsafe { lib::lilv_plugin_class_get_uri(inner) as _ })?,
            self.life.clone(),
        )
        .into()
    }

    /// # Panics
    /// Panics if the label could not be obtained.
    #[must_use]
    pub fn label(&self) -> Node {
        let _life = self.life.inner.read();
        let inner = self.inner.as_ptr();

        Node::new_borrowed(
            NonNull::new(unsafe { lib::lilv_plugin_class_get_label(inner) as _ }).unwrap(),
            self.life.clone(),
        )
    }

    #[must_use]
    pub fn children(&self) -> Option<PluginClasses> {
        let _life = self.life.inner.read();
        let inner = self.inner.as_ptr();
        PluginClasses {
            inner: NonNull::new(unsafe { lib::lilv_plugin_class_get_children(inner) })?,
            life: self.life.clone(),
        }
        .into()
    }
}

pub struct PluginClasses {
    pub(crate) inner: NonNull<lib::LilvPluginClasses>,
    pub(crate) life: Arc<Life>,
}

impl PluginClasses {
    #[must_use]
    pub fn iter(&self) -> PluginClassesIter {
        let _life = self.life.inner.read();
        PluginClassesIter {
            classes: self.inner.as_ptr(),
            iter: unsafe { lib::lilv_plugin_classes_begin(self.inner.as_ptr()) },
            life: self.life.clone(),
        }
    }

    #[must_use]
    pub fn size(&self) -> usize {
        let _life = self.life.inner.read();
        unsafe { lib::lilv_plugin_classes_size(self.inner.as_ptr()) as _ }
    }

    #[must_use]
    pub fn get_by_uri(&self, uri: &Node) -> Option<PluginClass> {
        let _life = self.life.inner.read();
        let inner = self.inner.as_ptr();
        let uri = uri.inner.as_ptr();

        Some(PluginClass::new_borrowed(
            NonNull::new(unsafe { lib::lilv_plugin_classes_get_by_uri(inner, uri) as _ })?,
            self.life.clone(),
        ))
    }
}

pub struct PluginClassesIter {
    classes: *mut lib::LilvPluginClasses,
    iter: *mut lib::LilvIter,
    life: Arc<Life>,
}

impl Iterator for PluginClassesIter {
    type Item = PluginClass;

    #[must_use]
    fn next(&mut self) -> Option<PluginClass> {
        let _life = self.life.inner.read();
        let ptr = unsafe { lib::lilv_plugin_classes_get(self.classes, self.iter) };
        if ptr.is_null() {
            None
        } else {
            self.iter = unsafe { lib::lilv_plugin_classes_next(self.classes, self.iter) };
            Some(PluginClass::new_borrowed(
                NonNull::new(ptr as _)?,
                self.life.clone(),
            ))
        }
    }
}
