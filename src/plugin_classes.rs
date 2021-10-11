use crate::node::Node;
use crate::plugin_class::PluginClass;
use crate::world::Life;
use lilv_sys as lib;
use std::ptr::NonNull;
use std::sync::Arc;

pub struct PluginClasses {
    pub(crate) inner: NonNull<lib::LilvPluginClasses>,
    pub(crate) owner: Arc<Life>,
}

impl PluginClasses {
    pub fn iter(&self) -> PluginClassesIter {
        PluginClassesIter {
            classes: self.inner.as_ptr(),
            iter: unsafe { lib::lilv_plugin_classes_begin(self.inner.as_ptr()) },
            owner: self.owner.clone(),
        }
    }

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
