use crate::node::Node;
use crate::nodes::Nodes;
use crate::plugin::Plugin;
use crate::scale_points::ScalePoints;
use crate::world::new_node;
use crate::world::ref_node;
use crate::Void;
use lilv_sys::*;
use std::ptr;

pub struct Port<'a> {
    pub(crate) port: *const LilvPort,
    pub(crate) plugin: &'a Plugin,
}

impl<'a> Port<'a> {
    pub fn get_node(&self) -> Node {
        ref_node(&self.plugin.world.clone(), unsafe {
            lilv_port_get_node(self.plugin.plugin, self.port)
        })
    }

    pub fn get_value(&self, predicate: &Node) -> Option<Nodes> {
        let nodes = unsafe { lilv_port_get_value(self.plugin.plugin, self.port, predicate.node) };
        if nodes.is_null() {
            None
        } else {
            Some(Nodes {
                nodes,
                world: self.plugin.world.clone(),
                owned: true,
            })
        }
    }

    pub fn get(&self, predicate: &Node) -> Option<Node> {
        let node = unsafe { lilv_port_get(self.plugin.plugin, self.port, predicate.node) };
        if node.is_null() {
            None
        } else {
            Some(new_node(&self.plugin.world.clone(), node))
        }
    }

    pub fn get_properties(&self) -> Nodes {
        Nodes {
            nodes: unsafe { lilv_port_get_properties(self.plugin.plugin, self.port) },
            world: self.plugin.world.clone(),
            owned: true,
        }
    }

    pub fn has_property(&self, property: &Node) -> bool {
        unsafe { lilv_port_has_property(self.plugin.plugin, self.port, property.node) }
    }

    pub fn supports_event(&self, event_type: &Node) -> bool {
        unsafe { lilv_port_supports_event(self.plugin.plugin, self.port, event_type.node) }
    }

    pub fn get_index(&self) -> u32 {
        unsafe { lilv_port_get_index(self.plugin.plugin, self.port) }
    }

    pub fn get_symbol(&self) -> Node {
        ref_node(&self.plugin.world, unsafe {
            lilv_port_get_symbol(self.plugin.plugin, self.port)
        })
    }

    pub fn get_name(&self) -> Node {
        new_node(&self.plugin.world, unsafe {
            lilv_port_get_name(self.plugin.plugin, self.port)
        })
    }

    pub fn get_classes(&self) -> Nodes {
        Nodes {
            nodes: unsafe { lilv_port_get_classes(self.plugin.plugin, self.port) as *mut Void },
            world: self.plugin.world.clone(),
            owned: false,
        }
    }

    pub fn get_range(&self) -> (Option<Node>, Option<Node>, Option<Node>) {
        let mut def = ptr::null_mut();
        let mut min = ptr::null_mut();
        let mut max = ptr::null_mut();
        unsafe { lilv_port_get_range(self.plugin.plugin, self.port, &mut def, &mut min, &mut max) };
        (
            if def.is_null() {
                None
            } else {
                Some(new_node(&self.plugin.world, def))
            },
            if min.is_null() {
                None
            } else {
                Some(new_node(&self.plugin.world, min))
            },
            if max.is_null() {
                None
            } else {
                Some(new_node(&self.plugin.world, max))
            },
        )
    }

    pub fn get_scale_points(&self) -> ScalePoints {
        ScalePoints {
            scale_points: unsafe { lilv_port_get_scale_points(self.plugin.plugin, self.port) },
            owned: true,
            world: self.plugin.world.clone(),
        }
    }

    pub fn is_a(&'a self, port_class: &'a Node) -> bool {
        unsafe { lilv_port_is_a((self.plugin).plugin, self.port, port_class.node) }
    }
}
