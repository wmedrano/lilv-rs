use crate::collection::Collection;
use crate::collection::Iter;
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

impl AsRef<*const Void> for PluginClasses {
    fn as_ref(&self) -> &*const Void {
        let ret: &*const LilvPluginClasses = &self.plugin_classes;
        unsafe { std::mem::transmute(ret) }
    }
}

impl<'a> Collection<'a> for PluginClasses
where
    Self: 'a,
{
    type Target = PluginClass;

    unsafe fn get(&self, i: *mut Void) -> Self::Target {
        PluginClass {
            plugin_class: lilv_plugin_classes_get(self.plugin_classes, i as *mut LilvIter)
                as *mut LilvPluginClass,
            world: self.world.clone(),
        }
    }
}

impl PluginClasses {
    pub fn get_by_uri<'a>(&'a self, uri: &Node) -> Option<PluginClass> {
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

    pub fn iter(&self) -> Iter<'_, Self> {
        Iter::new(
            self,
            lilv_plugin_classes_begin,
            lilv_plugin_classes_is_end,
            lilv_plugin_classes_next,
        )
    }

    pub fn size(&self) -> usize {
        unsafe { lilv_plugin_classes_size(self.plugin_classes) as usize }
    }
}

impl Drop for PluginClasses {
    fn drop(&mut self) {
        if self.owned {
            unsafe { lilv_plugin_classes_free(self.plugin_classes as *mut Void) }
        }
    }
}
