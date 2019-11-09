use crate::node::Node;
use crate::nodes::Nodes;
use crate::plugin_class::PluginClass;
use crate::port::Port;
use crate::uis::UIs;
use crate::world::new_node;
use crate::world::ref_node;
use crate::world::World;
use crate::Void;
use lilv_sys::*;
use std::ptr;
use std::rc::Rc;

pub struct Plugin {
    pub(crate) plugin: *const LilvPlugin,
    pub(crate) world: Rc<World>,
}

impl Plugin {
    pub fn verify(&self) -> bool {
        unsafe { lilv_plugin_verify(self.plugin) }
    }

    pub fn get_uri(&self) -> Node {
        ref_node(&self.world, unsafe { lilv_plugin_get_uri(self.plugin) })
    }

    pub fn get_bundle_uri(&self) -> Node {
        ref_node(&self.world, unsafe {
            lilv_plugin_get_bundle_uri(self.plugin)
        })
    }

    pub fn get_data_uris(&self) -> Nodes {
        Nodes {
            nodes: unsafe { lilv_plugin_get_data_uris(self.plugin) as *mut Void },
            world: self.world.clone(),
            owned: false,
        }
    }

    pub fn get_library_uri(&self) -> Node {
        ref_node(&self.world, unsafe {
            lilv_plugin_get_library_uri(self.plugin)
        })
    }

    pub fn get_name(&self) -> Node {
        new_node(&self.world, unsafe { lilv_plugin_get_name(self.plugin) })
    }

    pub fn get_class(&self) -> PluginClass {
        PluginClass {
            plugin_class: unsafe { lilv_plugin_get_class(self.plugin) as *mut LilvPluginClass },
            world: self.world.clone(),
        }
    }

    pub fn get_value(&self, predicate: &Node) -> Option<Nodes> {
        let nodes = unsafe { lilv_plugin_get_value(self.plugin, predicate.node) };
        if nodes.is_null() {
            None
        } else {
            Some(Nodes {
                nodes,
                world: self.world.clone(),
                owned: true,
            })
        }
    }

    pub fn has_feature(&self, feature: &Node) -> bool {
        unsafe { lilv_plugin_has_feature(self.plugin, feature.node) }
    }

    pub fn get_supported_features(&self) -> Nodes {
        Nodes {
            nodes: unsafe { lilv_plugin_get_supported_features(self.plugin) },
            world: self.world.clone(),
            owned: true,
        }
    }

    pub fn get_required_features(&self) -> Nodes {
        Nodes {
            nodes: unsafe { lilv_plugin_get_required_features(self.plugin) },
            world: self.world.clone(),
            owned: true,
        }
    }

    pub fn get_optional_features(&self) -> Nodes {
        Nodes {
            nodes: unsafe { lilv_plugin_get_optional_features(self.plugin) },
            world: self.world.clone(),
            owned: true,
        }
    }

    pub fn has_extension_data(&self, uri: &Node) -> bool {
        unsafe { lilv_plugin_has_extension_data(self.plugin, uri.node) }
    }

    pub fn get_extension_data(&self) -> Nodes {
        Nodes {
            nodes: unsafe { lilv_plugin_get_extension_data(self.plugin) },
            world: self.world.clone(),
            owned: true,
        }
    }

    pub fn get_num_ports(&self) -> u32 {
        unsafe { lilv_plugin_get_num_ports(self.plugin) }
    }

    pub fn get_num_ports_of_class<'a, T>(&self, classes: &[T]) -> u32
    where
        T: AsRef<Node<'a>>,
    {
        (0..self.get_num_ports())
            .filter(|p| {
                let port = self.get_port_by_index(*p).unwrap();
                classes.iter().all(|cls| port.is_a(cls.as_ref()))
            })
            .count() as u32
    }

    pub fn has_latency(&self) -> bool {
        unsafe { lilv_plugin_has_latency(self.plugin) }
    }

    pub fn get_latency_port_index(&self) -> u32 {
        unsafe { lilv_plugin_get_latency_port_index(self.plugin) }
    }

    pub fn get_port_by_index(&self, index: u32) -> Option<Port<'_>> {
        let ptr = unsafe { lilv_plugin_get_port_by_index(self.plugin, index) };
        if ptr.is_null() {
            None
        } else {
            Some(Port {
                port: ptr,
                plugin: self,
            })
        }
    }

    pub fn get_port_by_symbol<'a>(&'a self, symbol: &Node) -> Option<Port<'a>> {
        let ptr = unsafe { lilv_plugin_get_port_by_symbol(self.plugin, symbol.node) };
        if ptr.is_null() {
            None
        } else {
            Some(Port {
                port: ptr,
                plugin: self,
            })
        }
    }

    pub fn get_port_by_designation<'a, 'b, C>(
        &'a self,
        port_class: C,
        designation: &'b Node,
    ) -> Option<Port<'a>>
    where
        C: Into<Option<&'b Node<'b>>>,
    {
        let port_class = port_class.into().map_or(ptr::null(), |x| x.node);
        let ptr = unsafe {
            lilv_plugin_get_port_by_designation(self.plugin, port_class, designation.node)
        };
        if ptr.is_null() {
            None
        } else {
            Some(Port {
                port: ptr,
                plugin: self,
            })
        }
    }

    pub fn get_project(&self) -> Option<Node> {
        let node = unsafe { lilv_plugin_get_project(self.plugin) };
        if node.is_null() {
            None
        } else {
            Some(new_node(&self.world, node))
        }
    }

    pub fn get_author_name(&self) -> Option<Node> {
        let node = unsafe { lilv_plugin_get_author_name(self.plugin) };
        if node.is_null() {
            None
        } else {
            Some(new_node(&self.world, node))
        }
    }

    pub fn get_author_email(&self) -> Option<Node> {
        let node = unsafe { lilv_plugin_get_author_email(self.plugin) };
        if node.is_null() {
            None
        } else {
            Some(new_node(&self.world, node))
        }
    }

    pub fn get_author_homepage(&self) -> Option<Node> {
        let node = unsafe { lilv_plugin_get_author_homepage(self.plugin) };
        if node.is_null() {
            None
        } else {
            Some(new_node(&self.world, node))
        }
    }

    pub fn is_replaced(&self) -> bool {
        unsafe { lilv_plugin_is_replaced(self.plugin) }
    }

    pub fn get_related<T>(&self, tyep: &Node) -> Nodes {
        Nodes {
            nodes: unsafe { lilv_plugin_get_related(self.plugin, tyep.node) },
            world: self.world.clone(),
            owned: true,
        }
    }

    pub fn get_port_ranges_float<'a, Min, Max, Def>(
        &self,
        min_values: Min,
        max_values: Max,
        def_values: Def,
    ) -> Result<(), ()>
    where
        Min: Into<Option<&'a mut [f32]>>,
        Max: Into<Option<&'a mut [f32]>>,
        Def: Into<Option<&'a mut [f32]>>,
    {
        let min_values = min_values.into();
        let max_values = max_values.into();
        let def_values = def_values.into();

        let (equal_sizes, size) = match (&min_values, &max_values, &def_values) {
            (Some(a), Some(b), None) => (a.len() == b.len(), a.len()),
            (Some(a), None, Some(b)) => (a.len() == b.len(), a.len()),
            (None, Some(a), Some(b)) => (a.len() == b.len(), a.len()),
            (Some(a), Some(b), Some(c)) => (a.len() == b.len() && b.len() == c.len(), a.len()),
            _ => (true, self.get_num_ports() as usize),
        };

        if !equal_sizes || size != self.get_num_ports() as usize {
            return Err(());
        }

        let min_ptr = min_values.map_or(std::ptr::null_mut(), |x| x.as_mut_ptr());
        let max_ptr = max_values.map_or(std::ptr::null_mut(), |x| x.as_mut_ptr());
        let def_ptr = def_values.map_or(std::ptr::null_mut(), |x| x.as_mut_ptr());

        unsafe { lilv_plugin_get_port_ranges_float(self.plugin, min_ptr, max_ptr, def_ptr) };

        Ok(())
    }

    pub unsafe fn instantiate(
        &self,
        sample_rate: f64,
        features: *const *const lv2_raw::LV2Feature,
    ) -> Option<crate::Instance> {
        let ptr = lilv_plugin_instantiate(self.plugin, sample_rate, features);
        if ptr.is_null() {
            None
        } else {
            Some(crate::Instance::from_raw(ptr))
        }
    }

    pub fn get_uis(&self) -> UIs {
        UIs {
            uis: unsafe { lilv_plugin_get_uis(self.plugin) },
            owned: true,
            world: self.world.clone(),
        }
    }
}
