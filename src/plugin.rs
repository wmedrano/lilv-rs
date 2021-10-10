use crate::instance::Instance;
use crate::node::Node;
use crate::nodes::Nodes;
use crate::plugin_class::PluginClass;
use crate::port::Port;
use crate::uis::Uis;
use crate::world::InnerWorld;
use lilv_sys as lib;
use parking_lot::RwLock;
use std::ptr::NonNull;
use std::sync::Arc;

unsafe impl Send for Plugin {}
unsafe impl Sync for Plugin {}

/// Describes the ports of a plugin.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PortRanges {
    /// The minimum value of the port.
    pub min: f32,
    /// The maximum value of the port.
    pub max: f32,
    /// The default value of the port.
    pub default: f32,
}

/// Can be used to instantiave LV2 plugins.
pub struct Plugin {
    pub(crate) inner: RwLock<NonNull<lib::LilvPlugin>>,
    pub(crate) world: Arc<InnerWorld>,
}

impl Plugin {
    /// Returns true if the plugin is valid. If the world was created with
    /// `World::load_all`, then this is not necessary. Only valid plugins will
    /// have been loaded.
    pub fn verify(&self) -> bool {
        let plugin = self.inner.read().as_ptr();
        unsafe { lib::lilv_plugin_verify(plugin) }
    }

    /// The uri of the plugin.
    pub fn uri(&self) -> Node {
        let plugin = self.inner.read().as_ptr();

        Node::new_borrowed(
            NonNull::new(unsafe { lib::lilv_plugin_get_uri(plugin) as _ }).unwrap(),
            self.world.clone(),
        )
    }

    /// The uri of the plugin's bundle.
    pub fn bundle_uri(&self) -> Node {
        let plugin = self.inner.read().as_ptr();

        Node::new_borrowed(
            NonNull::new(unsafe { lib::lilv_plugin_get_bundle_uri(plugin) as _ }).unwrap(),
            self.world.clone(),
        )
    }

    /// The uri for the data.
    pub fn data_uris(&self) -> Nodes {
        let plugin = self.inner.read().as_ptr();

        Nodes {
            inner: NonNull::new(unsafe { lib::lilv_plugin_get_data_uris(plugin) as _ }).unwrap(),
            world: self.world.clone(),
        }
    }

    /// The uri for the library.
    pub fn library_uri(&self) -> Option<Node> {
        let plugin = self.inner.read().as_ptr();

        Some(Node::new_borrowed(
            NonNull::new(unsafe { lib::lilv_plugin_get_library_uri(plugin) as _ })?,
            self.world.clone(),
        ))
    }

    /// # Panics
    /// May panic if `verify()` returns false.
    pub fn name(&self) -> Node {
        let plugin = self.inner.read().as_ptr();

        Node::new(
            NonNull::new(unsafe { lib::lilv_plugin_get_name(plugin) as _ }).unwrap(),
            self.world.clone(),
        )
    }

    /// The class of the plugin.
    pub fn class(&self) -> PluginClass {
        let plugin = self.inner.read().as_ptr();

        PluginClass::new_borrowed(
            NonNull::new(unsafe { lib::lilv_plugin_get_class(plugin) as _ }).unwrap(),
            self.world.clone(),
        )
    }

    /// The value of the predicate or `None` if the plugin does not have one.
    pub fn value(&self, predicate: &Node) -> Option<Nodes> {
        let plugin = self.inner.read().as_ptr();
        let predicate = predicate.inner.read().as_ptr();

        Some(Nodes {
            inner: NonNull::new(unsafe { lib::lilv_plugin_get_value(plugin, predicate) })?,
            world: self.world.clone(),
        })
    }

    /// `true` if the plugin supports the feature.
    pub fn has_feature(&self, feature_uri: &Node) -> bool {
        let plugin = self.inner.read().as_ptr();
        let feature_uri = feature_uri.inner.read().as_ptr();

        unsafe { lib::lilv_plugin_has_feature(plugin, feature_uri) }
    }

    /// The set of features that are supported.
    pub fn supported_features(&self) -> Option<Nodes> {
        let plugin = self.inner.read().as_ptr();

        Some(Nodes::new(
            NonNull::new(unsafe { lib::lilv_plugin_get_supported_features(plugin) })?,
            self.world.clone(),
        ))
    }

    /// The set of features that are required to instantiate the plugin.
    pub fn required_features(&self) -> Option<Nodes> {
        let plugin = self.inner.read().as_ptr();

        Some(Nodes::new(
            NonNull::new(unsafe { lib::lilv_plugin_get_required_features(plugin) })?,
            self.world.clone(),
        ))
    }

    /// The set of features that are optional to instantiate the plugin.
    pub fn optional_features(&self) -> Option<Nodes> {
        let plugin = self.inner.read().as_ptr();

        Some(Nodes::new(
            NonNull::new(unsafe { lib::lilv_plugin_get_optional_features(plugin) })?,
            self.world.clone(),
        ))
    }

    /// True if the plugin has extension data for `uri`.
    pub fn has_extension_data(&self, uri: &Node) -> bool {
        let plugin = self.inner.read().as_ptr();
        let uri = uri.inner.read().as_ptr();

        unsafe { lib::lilv_plugin_has_extension_data(plugin, uri) }
    }

    pub fn extension_data(&self) -> Option<Nodes> {
        let plugin = self.inner.read().as_ptr();

        Some(Nodes::new(
            NonNull::new(unsafe { lib::lilv_plugin_get_extension_data(plugin) })?,
            self.world.clone(),
        ))
    }

    pub fn num_ports(&self) -> usize {
        let plugin = self.inner.read().as_ptr();
        unsafe { lib::lilv_plugin_get_num_ports(plugin) as _ }
    }

    /// Return the ranges for all ports.
    pub fn port_ranges_float(&self) -> Vec<PortRanges> {
        let ports_count = self.num_ports();
        let mut min = vec![0f32; ports_count];
        let mut max = vec![0f32; ports_count];
        let mut default = vec![0f32; ports_count];
        let plugin = self.inner.read().as_ptr();

        unsafe {
            lib::lilv_plugin_get_port_ranges_float(
                plugin,
                min.as_mut_ptr(),
                max.as_mut_ptr(),
                default.as_mut_ptr(),
            )
        };
        (0..ports_count)
            .map(|i| PortRanges {
                min: min[i],
                max: max[i],
                default: default[i],
            })
            .collect()
    }

    pub fn num_ports_of_class(&self, classes: &[&Node]) -> usize {
        (0..self.num_ports())
            .filter_map(|index| self.port_by_index(index))
            .filter(|port| classes.iter().all(|cls| port.is_a(cls)))
            .count()
    }

    pub fn has_latency(&self) -> bool {
        let plugin = self.inner.read().as_ptr();
        unsafe { lib::lilv_plugin_has_latency(plugin) }
    }

    pub fn latency_port_index(&self) -> Option<usize> {
        if self.has_latency() {
            let plugin = self.inner.read().as_ptr();
            Some(unsafe { lib::lilv_plugin_get_latency_port_index(plugin) as _ })
        } else {
            None
        }
    }

    pub fn port_by_index(&self, index: usize) -> Option<Port> {
        let plugin = self.inner.read().as_ptr();

        if index > std::u32::MAX as _ {
            return None;
        }

        Some(Port::new_borrowed(
            NonNull::new(unsafe { lib::lilv_plugin_get_port_by_index(plugin, index as _) as _ })?,
            self,
        ))
    }

    pub fn port_by_symbol(&self, symbol: &Node) -> Option<Port> {
        let plugin = self.inner.read().as_ptr();
        let symbol = symbol.inner.read().as_ptr();

        Some(Port::new_borrowed(
            NonNull::new(unsafe { lib::lilv_plugin_get_port_by_symbol(plugin, symbol) as _ })?,
            self,
        ))
    }

    pub fn port_by_designation(&self, port_class: &Node, designation: &Node) -> Option<Port> {
        let plugin = self.inner.read().as_ptr();
        let port_class = port_class.inner.read().as_ptr();
        let designation = designation.inner.read().as_ptr();

        Some(Port::new_borrowed(
            NonNull::new(unsafe {
                lib::lilv_plugin_get_port_by_designation(plugin, port_class, designation) as _
            })?,
            self,
        ))
    }

    pub fn project(&self) -> Option<Node> {
        let plugin = self.inner.read().as_ptr();

        Some(Node::new(
            NonNull::new(unsafe { lib::lilv_plugin_get_project(plugin) })?,
            self.world.clone(),
        ))
    }

    pub fn author_name(&self) -> Option<Node> {
        let plugin = self.inner.read().as_ptr();

        Some(Node::new(
            NonNull::new(unsafe { lib::lilv_plugin_get_author_name(plugin) })?,
            self.world.clone(),
        ))
    }

    pub fn author_email(&self) -> Option<Node> {
        let plugin = self.inner.read().as_ptr();

        Some(Node::new(
            NonNull::new(unsafe { lib::lilv_plugin_get_author_email(plugin) })?,
            self.world.clone(),
        ))
    }

    pub fn author_homepage(&self) -> Option<Node> {
        let plugin = self.inner.read().as_ptr();

        Some(Node::new(
            NonNull::new(unsafe { lib::lilv_plugin_get_author_homepage(plugin) })?,
            self.world.clone(),
        ))
    }

    pub fn is_replaced(&self) -> bool {
        let plugin = self.inner.read().as_ptr();
        unsafe { lib::lilv_plugin_is_replaced(plugin) }
    }

    // MAYBE TODO write_description

    // MAYBE TODO write_manifest_entry

    pub fn related(&self, tyep: Option<&Node>) -> Option<Nodes> {
        let plugin = self.inner.read().as_ptr();
        let tyep = tyep
            .map(|n| n.inner.read().as_ptr() as _)
            .unwrap_or(std::ptr::null());

        Some(Nodes::new(
            NonNull::new(unsafe { lib::lilv_plugin_get_related(plugin, tyep) })?,
            self.world.clone(),
        ))
    }

    pub fn uis(&self) -> Option<Uis<'_>> {
        let plugin = self.inner.read().as_ptr();

        Some(Uis {
            inner: NonNull::new(unsafe { lib::lilv_plugin_get_uis(plugin) })?,
            _world: self.world.clone(),
            plugin: self,
        })
    }

    /// # Safety
    /// Instantiating a plugin calls the plugin's code which itself may be
    /// unsafe.
    pub unsafe fn instantiate(
        &self,
        sample_rate: f64,
        features: *const *const lv2_raw::LV2Feature,
    ) -> Option<Instance> {
        let plugin = self.inner.read().as_ptr();

        Some(Instance {
            inner: NonNull::new(std::mem::transmute(lib::lilv_plugin_instantiate(
                plugin,
                sample_rate,
                std::mem::transmute(features),
            )))?,
        })
    }
}
