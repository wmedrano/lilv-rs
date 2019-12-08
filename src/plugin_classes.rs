use crate::node::Node;
use crate::plugin_class::PluginClass;
use crate::world::World;
use crate::Void;
use lilv_sys::*;
use std::rc::Rc;

pub struct PluginClasses {
    pub(crate) plugin_classes: *const LilvPluginClasses,
    pub(crate) owned: bool,
    pub(crate) world: Rc<World>,
}

impl PluginClasses {
    pub fn by_uri<'a>(&'a self, uri: &Node) -> Option<PluginClass> {
        let ptr = unsafe { lilv_plugin_classes_get_by_uri(self.plugin_classes, uri.node) };
        if ptr.is_null() {
            None
        } else {
            Some(PluginClass {
                plugin_class: ptr as *mut LilvPluginClass,
                world: self.world.clone(),
            })
        }
    }

    /// Iterator over all plugin classes.
    pub fn iter(&self) -> PluginClassesIter<'_> {
        PluginClassesIter {
            plugin_classes: self,
            iter: unsafe { lilv_plugin_classes_begin(self.plugin_classes) },
        }
    }

    /// The number of plugin classes.
    pub fn len(&self) -> usize {
        unsafe { lilv_plugin_classes_size(self.plugin_classes) as usize }
    }

    /// Returns true if there are no plugin classes.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Drop for PluginClasses {
    fn drop(&mut self) {
        if self.owned {
            unsafe { lilv_plugin_classes_free(self.plugin_classes as *mut Void) }
        }
    }
}

pub struct PluginClassesIter<'a> {
    plugin_classes: &'a PluginClasses,
    iter: *mut LilvIter,
}

impl<'a> Iterator for PluginClassesIter<'a> {
    type Item = PluginClass;

    fn next(&mut self) -> Option<PluginClass> {
        let ptr = unsafe { lilv_plugin_classes_get(self.plugin_classes.plugin_classes, self.iter) };
        if ptr.is_null() {
            None
        } else {
            self.iter =
                unsafe { lilv_plugin_classes_next(self.plugin_classes.plugin_classes, self.iter) };
            Some(PluginClass {
                plugin_class: ptr as *mut LilvPluginClass,
                world: self.plugin_classes.world.clone(),
            })
        }
    }
}
