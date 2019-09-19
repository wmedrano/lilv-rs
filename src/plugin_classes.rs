use crate::collection::Collection;
use crate::collection::Iter;
use crate::node::Node;
use crate::plugin_class::PluginClass;
use crate::world::World;
use crate::Void;
use std::rc::Rc;

#[link(name = "lilv-0")]
extern "C" {
    fn lilv_plugin_classes_free(plugin_classes: *mut Void);
    fn lilv_plugin_classes_size(plugin_classes: *const Void) -> u32;
    fn lilv_plugin_classes_get(plugin_classes: *const Void, i: *mut Void) -> *const Void;
    fn lilv_plugin_classes_begin(plugin_classes: *const Void) -> *mut Void;
    fn lilv_plugin_classes_next(plugin_classes: *const Void, i: *mut Void) -> *mut Void;
    fn lilv_plugin_classes_is_end(plugin_classes: *const Void, i: *mut Void) -> u8;
    fn lilv_plugin_classes_get_by_uri(plugin_classes: *const Void, uri: *const Void)
        -> *const Void;
}

pub struct PluginClasses {
    pub(crate) plugin_classes: *const Void,
    pub(crate) owned: bool,
    pub(crate) world: Rc<World>,
}

impl AsRef<*const Void> for PluginClasses {
    fn as_ref(&self) -> &*const Void {
        &self.plugin_classes
    }
}

impl<'a> Collection<'a> for PluginClasses
where
    Self: 'a,
{
    type Target = PluginClass;

    fn get(&self, i: *mut Void) -> Self::Target {
        PluginClass {
            plugin_class: unsafe { lilv_plugin_classes_get(self.plugin_classes, i) } as *mut Void,
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
                plugin_class: ptr as *mut Void,
                world: self.world.clone(),
            })
        }
    }

    pub fn iter<'a>(&'a self) -> Iter<'a, Self> {
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
