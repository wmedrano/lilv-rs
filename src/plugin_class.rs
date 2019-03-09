use crate::node::Node;
use crate::node::Uri;
use crate::plugin_classes::PluginClasses;
use crate::world::ref_node;
use crate::world::World;
use crate::Void;
use std::rc::Rc;

#[link(name = "lilv-0")]
extern "C" {
    fn lilv_plugin_class_get_parent_uri(plugin_class: *const Void) -> *const Void;
    fn lilv_plugin_class_get_uri(plugin_class: *const Void) -> *const Void;
    fn lilv_plugin_class_get_label(plugin_class: *const Void) -> *const Void;
    fn lilv_plugin_class_get_children(plugin_class: *const Void) -> *mut Void;
}

pub struct PluginClass {
    pub(crate) plugin_class: *mut Void,
    pub(crate) world: Rc<World>,
}

impl PluginClass {
    pub fn get_parent_uri(&self) -> Node<Uri> {
        ref_node(&self.world, unsafe {
            lilv_plugin_class_get_parent_uri(self.plugin_class)
        })
    }

    pub fn get_uri(&self) -> Node<Uri> {
        ref_node(&self.world, unsafe {
            lilv_plugin_class_get_uri(self.plugin_class)
        })
    }

    pub fn get_label(&self) -> Node<String> {
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
