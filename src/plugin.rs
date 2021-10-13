use crate::instance::Instance;
use crate::node::{Node, Nodes};
use crate::plugin_class::PluginClass;
use crate::port::Port;
use crate::uis::Uis;
use crate::world::Life;
use lilv_sys as lib;
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
    pub(crate) inner: NonNull<lib::LilvPlugin>,
    pub(crate) life: Arc<Life>,
}

impl Plugin {
    /// Returns true if the plugin is valid. If the world was created with
    /// `World::load_all`, then this is not necessary. Only valid plugins will
    /// have been loaded.
    #[must_use]
    pub fn verify(&self) -> bool {
        let _life = self.life.inner.lock();
        let plugin = self.inner.as_ptr();
        unsafe { lib::lilv_plugin_verify(plugin) }
    }

    /// The uri of the plugin.
    /// # Panics
    /// Panics if the uri could not be obtained.
    #[must_use]
    pub fn uri(&self) -> Node {
        let _life = self.life.inner.lock();
        let plugin = self.inner.as_ptr();

        {
            let ptr = NonNull::new(unsafe { lib::lilv_plugin_get_uri(plugin) as _ }).unwrap();
            let world = self.life.clone();
            Node {
                inner: ptr,
                borrowed: true,
                life: world,
            }
        }
    }

    /// The uri of the plugin's bundle.
    /// # Panics
    /// Panics if the bundle uri could not be obtained.
    #[must_use]
    pub fn bundle_uri(&self) -> Node {
        let _life = self.life.inner.lock();
        let plugin = self.inner.as_ptr();

        {
            let ptr =
                NonNull::new(unsafe { lib::lilv_plugin_get_bundle_uri(plugin) as _ }).unwrap();
            let world = self.life.clone();
            Node {
                inner: ptr,
                borrowed: true,
                life: world,
            }
        }
    }

    /// The uri for the data.
    /// # Panics
    /// Panics if the `data_uris` could not be obtained.
    #[must_use]
    pub fn data_uris(&self) -> Nodes {
        let _life = self.life.inner.lock();
        let plugin = self.inner.as_ptr();

        Nodes {
            inner: NonNull::new(unsafe { lib::lilv_plugin_get_data_uris(plugin) as _ }).unwrap(),
            life: self.life.clone(),
        }
    }

    /// The uri for the library.
    #[must_use]
    pub fn library_uri(&self) -> Option<Node> {
        let _life = self.life.inner.lock();
        let plugin = self.inner.as_ptr();

        Some({
            let ptr = NonNull::new(unsafe { lib::lilv_plugin_get_library_uri(plugin) as _ })?;
            let world = self.life.clone();
            Node {
                inner: ptr,
                borrowed: true,
                life: world,
            }
        })
    }

    /// # Panics
    /// May panic if `verify()` returns false.
    #[must_use]
    pub fn name(&self) -> Node {
        let _life = self.life.inner.lock();
        let plugin = self.inner.as_ptr();

        {
            let ptr = NonNull::new(unsafe { lib::lilv_plugin_get_name(plugin).cast() }).unwrap();
            let world = self.life.clone();
            Node {
                inner: ptr,
                borrowed: false,
                life: world,
            }
        }
    }

    /// The class of the plugin.
    /// # Panics
    /// Panics if the pluginc class could not be found.
    #[must_use]
    pub fn class(&self) -> PluginClass {
        let _life = self.life.inner.lock();
        let plugin = self.inner.as_ptr();

        PluginClass::new_borrowed(
            NonNull::new(unsafe { lib::lilv_plugin_get_class(plugin) as _ }).unwrap(),
            self.life.clone(),
        )
    }

    /// The value of the predicate or `None` if the plugin does not have one.
    #[must_use]
    pub fn value(&self, predicate: &Node) -> Option<Nodes> {
        let _life = self.life.inner.lock();
        let plugin = self.inner.as_ptr();
        let predicate = predicate.inner.as_ptr();

        Some(Nodes {
            inner: NonNull::new(unsafe { lib::lilv_plugin_get_value(plugin, predicate) })?,
            life: self.life.clone(),
        })
    }

    /// `true` if the plugin supports the feature.
    #[must_use]
    pub fn has_feature(&self, feature_uri: &Node) -> bool {
        let _life = self.life.inner.lock();
        let plugin = self.inner.as_ptr();
        let feature_uri = feature_uri.inner.as_ptr();

        unsafe { lib::lilv_plugin_has_feature(plugin, feature_uri) }
    }

    /// The set of features that are supported.
    #[must_use]
    pub fn supported_features(&self) -> Option<Nodes> {
        let _life = self.life.inner.lock();
        let plugin = self.inner.as_ptr();

        Some(Nodes::new(
            NonNull::new(unsafe { lib::lilv_plugin_get_supported_features(plugin) })?,
            self.life.clone(),
        ))
    }

    /// The set of features that are required to instantiate the plugin.
    #[must_use]
    pub fn required_features(&self) -> Option<Nodes> {
        let _life = self.life.inner.lock();
        let plugin = self.inner.as_ptr();

        Some(Nodes::new(
            NonNull::new(unsafe { lib::lilv_plugin_get_required_features(plugin) })?,
            self.life.clone(),
        ))
    }

    /// The set of features that are optional to instantiate the plugin.
    #[must_use]
    pub fn optional_features(&self) -> Option<Nodes> {
        let _life = self.life.inner.lock();
        let plugin = self.inner.as_ptr();

        Some(Nodes::new(
            NonNull::new(unsafe { lib::lilv_plugin_get_optional_features(plugin) })?,
            self.life.clone(),
        ))
    }

    /// True if the plugin has extension data for `uri`.
    #[must_use]
    pub fn has_extension_data(&self, uri: &Node) -> bool {
        let _life = self.life.inner.lock();
        let plugin = self.inner.as_ptr();
        let uri = uri.inner.as_ptr();

        unsafe { lib::lilv_plugin_has_extension_data(plugin, uri) }
    }

    #[must_use]
    pub fn extension_data(&self) -> Option<Nodes> {
        let _life = self.life.inner.lock();
        let plugin = self.inner.as_ptr();

        Some(Nodes::new(
            NonNull::new(unsafe { lib::lilv_plugin_get_extension_data(plugin) })?,
            self.life.clone(),
        ))
    }

    #[must_use]
    pub fn num_ports(&self) -> usize {
        let _life = self.life.inner.lock();
        let plugin = self.inner.as_ptr();
        unsafe { lib::lilv_plugin_get_num_ports(plugin) as _ }
    }

    /// Return the ranges for all ports.
    #[must_use]
    pub fn port_ranges_float(&self) -> Vec<PortRanges> {
        let _life = self.life.inner.lock();
        let ports_count = self.num_ports();
        let mut min = vec![0_f32; ports_count];
        let mut max = vec![0_f32; ports_count];
        let mut default = vec![0_f32; ports_count];
        let plugin = self.inner.as_ptr();

        unsafe {
            lib::lilv_plugin_get_port_ranges_float(
                plugin,
                min.as_mut_ptr(),
                max.as_mut_ptr(),
                default.as_mut_ptr(),
            );
        }
        (0..ports_count)
            .map(|i| PortRanges {
                min: min[i],
                max: max[i],
                default: default[i],
            })
            .collect()
    }

    #[must_use]
    pub fn num_ports_of_class(&self, classes: &[&Node]) -> usize {
        let _life = self.life.inner.lock();
        (0..self.num_ports())
            .filter_map(|index| self.port_by_index(index))
            .filter(|port| classes.iter().all(|cls| port.is_a(cls)))
            .count()
    }

    #[must_use]
    pub fn has_latency(&self) -> bool {
        let _life = self.life.inner.lock();
        let plugin = self.inner.as_ptr();
        unsafe { lib::lilv_plugin_has_latency(plugin) }
    }

    #[must_use]
    pub fn latency_port_index(&self) -> Option<usize> {
        let _life = self.life.inner.lock();
        if self.has_latency() {
            let plugin = self.inner.as_ptr();
            Some(unsafe { lib::lilv_plugin_get_latency_port_index(plugin) as _ })
        } else {
            None
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    #[must_use]
    pub fn port_by_index(&self, index: usize) -> Option<Port> {
        let _life = self.life.inner.lock();
        let plugin = self.inner.as_ptr();

        if index > std::u32::MAX as _ {
            return None;
        }

        Some({
            let inner = NonNull::new(unsafe {
                lib::lilv_plugin_get_port_by_index(plugin, index as _) as _
            })?;
            Port {
                inner,
                plugin: self,
            }
        })
    }

    #[must_use]
    pub fn port_by_symbol(&self, symbol: &Node) -> Option<Port> {
        let _life = self.life.inner.lock();
        let plugin = self.inner.as_ptr();
        let symbol = symbol.inner.as_ptr();

        Some({
            let inner =
                NonNull::new(unsafe { lib::lilv_plugin_get_port_by_symbol(plugin, symbol) as _ })?;
            Port {
                inner,
                plugin: self,
            }
        })
    }

    #[must_use]
    pub fn port_by_designation(&self, port_class: &Node, designation: &Node) -> Option<Port> {
        let _life = self.life.inner.lock();
        let plugin = self.inner.as_ptr();
        let port_class = port_class.inner.as_ptr();
        let designation = designation.inner.as_ptr();

        Some({
            let inner = NonNull::new(unsafe {
                lib::lilv_plugin_get_port_by_designation(plugin, port_class, designation) as _
            })?;
            Port {
                inner,
                plugin: self,
            }
        })
    }

    #[must_use]
    pub fn project(&self) -> Option<Node> {
        let _life = self.life.inner.lock();
        let plugin = self.inner.as_ptr();

        Some({
            let ptr = NonNull::new(unsafe { lib::lilv_plugin_get_project(plugin) })?;
            let world = self.life.clone();
            Node {
                inner: ptr,
                borrowed: false,
                life: world,
            }
        })
    }

    #[must_use]
    pub fn author_name(&self) -> Option<Node> {
        let _life = self.life.inner.lock();
        let plugin = self.inner.as_ptr();

        Some({
            let ptr = NonNull::new(unsafe { lib::lilv_plugin_get_author_name(plugin) })?;
            let world = self.life.clone();
            Node {
                inner: ptr,
                borrowed: false,
                life: world,
            }
        })
    }

    #[must_use]
    pub fn author_email(&self) -> Option<Node> {
        let _life = self.life.inner.lock();
        let plugin = self.inner.as_ptr();

        Some({
            let ptr = NonNull::new(unsafe { lib::lilv_plugin_get_author_email(plugin) })?;
            let world = self.life.clone();
            Node {
                inner: ptr,
                borrowed: false,
                life: world,
            }
        })
    }

    #[must_use]
    pub fn author_homepage(&self) -> Option<Node> {
        let _life = self.life.inner.lock();
        let plugin = self.inner.as_ptr();

        Some({
            let ptr = NonNull::new(unsafe { lib::lilv_plugin_get_author_homepage(plugin) })?;
            let world = self.life.clone();
            Node {
                inner: ptr,
                borrowed: false,
                life: world,
            }
        })
    }

    #[must_use]
    pub fn is_replaced(&self) -> bool {
        let _life = self.life.inner.lock();
        let plugin = self.inner.as_ptr();
        unsafe { lib::lilv_plugin_is_replaced(plugin) }
    }

    // MAYBE TODO write_description

    // MAYBE TODO write_manifest_entry

    #[must_use]
    pub fn related(&self, typ: Option<&Node>) -> Option<Nodes> {
        let _life = self.life.inner.lock();
        let plugin = self.inner.as_ptr();
        let plugin_type = typ.map_or(std::ptr::null(), |n| n.inner.as_ptr() as _);

        Some(Nodes::new(
            NonNull::new(unsafe { lib::lilv_plugin_get_related(plugin, plugin_type) })?,
            self.life.clone(),
        ))
    }

    #[must_use]
    pub fn uis(&self) -> Option<Uis<'_>> {
        let _life = self.life.inner.lock();
        let plugin = self.inner.as_ptr();

        Some(Uis {
            inner: NonNull::new(unsafe { lib::lilv_plugin_get_uis(plugin) })?,
            life: self.life.clone(),
            plugin: self,
        })
    }

    /// # Safety
    /// Instantiating a plugin calls the plugin's code which itself may be
    /// unsafe.
    #[must_use]
    pub unsafe fn instantiate(
        &self,
        sample_rate: f64,
        features: *const *const lv2_raw::LV2Feature,
    ) -> Option<Instance> {
        let _life = self.life.inner.lock();
        let plugin = self.inner.as_ptr();

        Some(Instance {
            inner: NonNull::new(
                (lib::lilv_plugin_instantiate(plugin, sample_rate, std::mem::transmute(features)))
                    .cast(),
            )?,
        })
    }
}

/// An iterator over plugins.
pub struct PluginsIter {
    pub(crate) life: Arc<Life>,
    pub(crate) ptr: *const lib::LilvPlugins,
    pub(crate) iter: *mut lib::LilvIter,
}

impl Iterator for PluginsIter {
    type Item = Plugin;

    fn next(&mut self) -> Option<Plugin> {
        let _life = self.life.inner.lock();
        let ptr: *mut lib::LilvPlugin =
            unsafe { lib::lilv_plugins_get(self.ptr, self.iter) } as *mut _;
        self.iter = unsafe { lib::lilv_plugins_next(self.ptr, self.iter) };
        match NonNull::new(ptr) {
            Some(ptr) => Some(Plugin {
                life: self.life.clone(),
                inner: ptr,
            }),
            None => None,
        }
    }
}
