use crate::instance::Instance;
use crate::instance::InstanceImpl;
use crate::node::Node;
use crate::node::Uri;
use crate::nodes::Nodes;
use crate::port::Port;
use crate::world::ref_node;
use crate::world::World;
use crate::Void;
use std::marker::PhantomData;
use std::rc::Rc;

#[link(name = "lilv-0")]
extern "C" {
    fn lilv_plugin_get_uri(plugin: *const Void) -> *const Void;
    fn lilv_plugin_get_bundle_uri(plugin: *const Void) -> *const Void;
    fn lilv_plugin_get_data_uris(plugin: *const Void) -> *const Void;
    fn lilv_plugin_get_num_ports(plugin: *const Void) -> u32;
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

    pub fn get_num_ports(&self) -> u32 {
        unsafe { lilv_plugin_get_num_ports(self.plugin) }
    }

    pub fn get_port_by_index<'b>(&'b self, index: u32) -> Option<Port<'b>> {
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

    pub fn get_port_ranges_float<'b, Min, Max, Def>(
        &self,
        min_values: Min,
        max_values: Max,
        def_values: Def,
    ) -> Result<(), ()>
    where
        Min: Into<Option<&'b mut [f32]>>,
        Max: Into<Option<&'b mut [f32]>>,
        Def: Into<Option<&'b mut [f32]>>,
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
