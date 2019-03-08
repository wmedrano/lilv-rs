use crate::instance::Instance;
use crate::instance::InstanceImpl;
use crate::node::Any;
use crate::node::Node;
use crate::node::Uri;
use crate::nodes::Nodes;
use crate::port::Port;
use crate::world::new_node;
use crate::world::ref_node;
use crate::world::World;
use crate::Void;
use std::marker::PhantomData;
use std::ptr;
use std::rc::Rc;

#[link(name = "lilv-0")]
extern "C" {
    fn lilv_plugin_get_uri(plugin: *const Void) -> *const Void;
    fn lilv_plugin_get_bundle_uri(plugin: *const Void) -> *const Void;
    fn lilv_plugin_get_data_uris(plugin: *const Void) -> *const Void;
    fn lilv_plugin_get_library_uri(plugin: *const Void) -> *const Void;
    fn lilv_plugin_get_name(plugin: *const Void) -> *mut Void;
    fn lilv_plugin_get_value(plugin: *const Void, predicate: *const Void) -> *mut Void;
    fn lilv_plugin_has_feature(plugin: *const Void, feature: *const Void) -> u8;
    fn lilv_plugin_get_supported_features(plugin: *const Void) -> *mut Void;
    fn lilv_plugin_get_required_features(plugin: *const Void) -> *mut Void;
    fn lilv_plugin_get_optional_features(plugin: *const Void) -> *mut Void;
    fn lilv_plugin_has_extension_data(plugin: *const Void, uri: *const Void) -> u8;
    fn lilv_plugin_get_extension_data(plugin: *const Void) -> *mut Void;
    fn lilv_plugin_get_num_ports(plugin: *const Void) -> u32;
    fn lilv_plugin_has_latency(plugin: *const Void) -> u8;
    fn lilv_plugin_get_latency_port_index(plugin: *const Void) -> u32;
    fn lilv_plugin_get_port_by_symbol(plugin: *const Void, symbol: *const Void) -> *const Void;
    fn lilv_plugin_get_port_by_designation(
        plugin: *const Void,
        port_class: *const Void,
        designation: *const Void,
    ) -> *const Void;
    fn lilv_plugin_get_project(plugin: *const Void) -> *mut Void;
    fn lilv_plugin_get_author_name(plugin: *const Void) -> *mut Void;
    fn lilv_plugin_get_author_email(plugin: *const Void) -> *mut Void;
    fn lilv_plugin_get_author_homepage(plugin: *const Void) -> *mut Void;
    fn lilv_plugin_is_replaced(plugin: *const Void) -> u8;
    fn lilv_plugin_get_related(plugin: *const Void, tyep: *const Void) -> *mut Void;
    fn lilv_plugin_get_port_ranges_float(
        plugin: *const Void,
        min_values: *mut f32,
        max_values: *mut f32,
        def_values: *mut f32,
    );
    fn lilv_plugin_get_port_by_index(plugin: *const Void, index: u32) -> *const Void;
    fn lilv_plugin_instantiate(
        plugin: *const Void,
        sample_rate: f64,
        features: *const *const lv2_raw::LV2Feature,
    ) -> *mut InstanceImpl;
}

pub struct Plugin {
    pub(crate) plugin: *const Void,
    pub(crate) world: Rc<World>,
}

impl Plugin {
    pub fn get_uri(&self) -> Node<Uri> {
        ref_node(&self.world, unsafe { lilv_plugin_get_uri(self.plugin) })
    }

    pub fn get_bundle_uri(&self) -> Node<Uri> {
        ref_node(&self.world, unsafe {
            lilv_plugin_get_bundle_uri(self.plugin)
        })
    }

    pub fn get_data_uris(&self) -> Nodes<Uri> {
        Nodes {
            nodes: unsafe { lilv_plugin_get_data_uris(self.plugin) as *mut Void },
            world: self.world.clone(),
            owned: false,
            _phantom: PhantomData,
        }
    }

    pub fn get_library_uri(&self) -> Node<Uri> {
        ref_node(&self.world, unsafe {
            lilv_plugin_get_library_uri(self.plugin)
        })
    }

    pub fn get_name(&self) -> Node<crate::node::String> {
        new_node(&self.world, unsafe { lilv_plugin_get_name(self.plugin) })
    }

    pub fn get_value(&self, predicate: &Node<Any>) -> Option<Nodes<Any>> {
        let nodes = unsafe { lilv_plugin_get_value(self.plugin, predicate.node) };
        if nodes.is_null() {
            None
        } else {
            Some(Nodes {
                nodes,
                world: self.world.clone(),
                owned: true,
                _phantom: PhantomData,
            })
        }
    }

    pub fn has_feature(&self, feature: &Node<Uri>) -> bool {
        unsafe { lilv_plugin_has_feature(self.plugin, feature.node) != 0 }
    }

    pub fn get_supported_features(&self) -> Nodes<Uri> {
        Nodes {
            nodes: unsafe { lilv_plugin_get_supported_features(self.plugin) },
            world: self.world.clone(),
            owned: true,
            _phantom: PhantomData,
        }
    }

    pub fn get_required_features(&self) -> Nodes<Uri> {
        Nodes {
            nodes: unsafe { lilv_plugin_get_required_features(self.plugin) },
            world: self.world.clone(),
            owned: true,
            _phantom: PhantomData,
        }
    }

    pub fn get_optional_features(&self) -> Nodes<Uri> {
        Nodes {
            nodes: unsafe { lilv_plugin_get_optional_features(self.plugin) },
            world: self.world.clone(),
            owned: true,
            _phantom: PhantomData,
        }
    }

    pub fn has_extension_data(&self, uri: &Node<Uri>) -> bool {
        unsafe { lilv_plugin_has_extension_data(self.plugin, uri.node) != 0 }
    }

    pub fn get_extension_data(&self) -> Nodes<Uri> {
        Nodes {
            nodes: unsafe { lilv_plugin_get_extension_data(self.plugin) },
            world: self.world.clone(),
            owned: true,
            _phantom: PhantomData,
        }
    }

    pub fn get_num_ports(&self) -> u32 {
        unsafe { lilv_plugin_get_num_ports(self.plugin) }
    }

    pub fn has_latency(&self) -> bool {
        unsafe { lilv_plugin_has_latency(self.plugin) != 0 }
    }

    pub fn get_latency_port_index(&self) -> u32 {
        unsafe { lilv_plugin_get_latency_port_index(self.plugin) }
    }

    pub fn get_port_by_index<'a>(&'a self, index: u32) -> Option<Port<'a>> {
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

    pub fn get_port_by_symbol<'a>(
        &'a self,
        symbol: &Node<crate::node::String>,
    ) -> Option<Port<'a>> {
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
        designation: &'b Node<Uri>,
    ) -> Option<Port<'a>>
    where
        C: Into<Option<&'b Node<Uri>>>,
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

    pub fn get_project(&self) -> Option<Node<Any>> {
        let node = unsafe { lilv_plugin_get_project(self.plugin) };
        if node.is_null() {
            None
        } else {
            Some(new_node(&self.world, node))
        }
    }

    pub fn get_author_name(&self) -> Option<Node<crate::node::String>> {
        let node = unsafe { lilv_plugin_get_author_name(self.plugin) };
        if node.is_null() {
            None
        } else {
            Some(new_node(&self.world, node))
        }
    }

    pub fn get_author_email(&self) -> Option<Node<Any>> {
        let node = unsafe { lilv_plugin_get_author_email(self.plugin) };
        if node.is_null() {
            None
        } else {
            Some(new_node(&self.world, node))
        }
    }

    pub fn get_author_homepage(&self) -> Option<Node<Any>> {
        let node = unsafe { lilv_plugin_get_author_homepage(self.plugin) };
        if node.is_null() {
            None
        } else {
            Some(new_node(&self.world, node))
        }
    }

    pub fn is_replaced(&self) -> bool {
        unsafe { lilv_plugin_is_replaced(self.plugin) != 0 }
    }

    pub fn get_related<'a, T>(&self, tyep: T) -> Nodes<Any>
    where
        T: Into<Option<&'a Node<Any>>>,
    {
        let tyep = tyep.into().map_or(ptr::null(), |x| x.node);
        Nodes {
            nodes: unsafe { lilv_plugin_get_related(self.plugin, tyep) },
            world: self.world.clone(),
            owned: true,
            _phantom: PhantomData,
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
    ) -> Option<Instance> {
        let ptr = lilv_plugin_instantiate(self.plugin, sample_rate, features);
        if ptr.is_null() {
            None
        } else {
            Some(Instance(ptr))
        }
    }
}
