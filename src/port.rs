use crate::node::Node;
use crate::node::Uri;
use crate::plugin::Plugin;
use crate::Void;

#[link(name = "lilv-0")]
extern "C" {
    fn lilv_port_is_a(plugin: *const Void, port: *const Void, port_class: *const Void) -> u8;
}

pub struct Port<'a> {
    pub(crate) port: *const Void,
    pub(crate) plugin: &'a Plugin,
}

impl<'a> Port<'a> {
    pub fn is_a(&'a self, port_class: &'a Node<Uri>) -> bool {
        unsafe { lilv_port_is_a((self.plugin).plugin, self.port, port_class.node) != 0 }
    }
}
