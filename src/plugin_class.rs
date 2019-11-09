use crate::node::Node;
use crate::plugin_classes::PluginClasses;
use crate::world::ref_node;
use crate::world::World;
use lilv_sys::*;
use std::rc::Rc;

pub struct PluginClass {
    pub(crate) plugin_class: *mut LilvPluginClass,
    pub(crate) world: Rc<World>,
}

impl PluginClass {
    pub fn get_parent_uri(&self) -> Node {
        ref_node(&self.world, unsafe {
            lilv_plugin_class_get_parent_uri(self.plugin_class)
        })
    }

    pub fn get_uri(&self) -> Node {
        ref_node(&self.world, unsafe {
            lilv_plugin_class_get_uri(self.plugin_class)
        })
    }

    pub fn get_label(&self) -> Node {
        ref_node(&self.world, unsafe {
            lilv_plugin_class_get_label(self.plugin_class)
        })
    }

    pub fn get_children(&self) -> PluginClasses {
        PluginClasses {
            plugin_classes: unsafe { lilv_plugin_class_get_children(self.plugin_class) },
            owned: true,
            world: self.world.clone(),
        }
    }
}
