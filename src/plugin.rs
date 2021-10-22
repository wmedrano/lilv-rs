use crate::instance::Instance;
use crate::node::{Node, Nodes};
use crate::port::{FloatRanges, Port};
use crate::ui::Uis;
use crate::world::Life;
use lilv_sys as lib;
use std::borrow::Borrow;
use std::convert::TryFrom;
use std::ptr::NonNull;
use std::sync::Arc;

unsafe impl Send for Plugin {}
unsafe impl Sync for Plugin {}

/// Can be used to instantiave LV2 plugins.
#[derive(Clone)]
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
        let ptr = NonNull::new(unsafe { lib::lilv_plugin_get_uri(plugin) as _ }).unwrap();
        let world = self.life.clone();
        Node {
            inner: ptr,
            borrowed: true,
            life: world,
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
    #[must_use]
    pub fn data_uris(&self) -> Nodes {
        let _life = self.life.inner.lock();
        let plugin = self.inner.as_ptr();

        Nodes {
            inner: unsafe { lib::lilv_plugin_get_data_uris(plugin) },
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
            NonNull::new(unsafe { lib::lilv_plugin_get_class(plugin) as *mut _ }).unwrap(),
            self.life.clone(),
        )
    }

    /// The value of the predicate. `Nodes` may be empty if the plugin does not
    /// have one.
    #[must_use]
    pub fn value(&self, predicate: &Node) -> Nodes {
        let _life = self.life.inner.lock();
        let plugin = self.inner.as_ptr();
        let predicate = predicate.inner.as_ptr();

        Nodes {
            inner: unsafe { lib::lilv_plugin_get_value(plugin, predicate) },
            life: self.life.clone(),
        }
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
    pub fn supported_features(&self) -> Nodes {
        let _life = self.life.inner.lock();
        let plugin = self.inner.as_ptr();
        let inner = unsafe { lib::lilv_plugin_get_supported_features(plugin) };
        let world = self.life.clone();
        Nodes { inner, life: world }
    }

    /// The set of features that are required to instantiate the plugin.
    #[must_use]
    pub fn required_features(&self) -> Nodes {
        let _life = self.life.inner.lock();
        let plugin = self.inner.as_ptr();
        let inner = unsafe { lib::lilv_plugin_get_required_features(plugin) };
        let world = self.life.clone();
        Nodes { inner, life: world }
    }

    /// The set of features that are optional to instantiate the plugin.
    #[must_use]
    pub fn optional_features(&self) -> Nodes {
        let _life = self.life.inner.lock();
        let plugin = self.inner.as_ptr();
        let inner = unsafe { lib::lilv_plugin_get_optional_features(plugin) };
        let world = self.life.clone();
        Nodes { inner, life: world }
    }

    /// Returns `true` if the plugin has extension data for `uri`.
    #[must_use]
    pub fn has_extension_data(&self, uri: &Node) -> bool {
        let _life = self.life.inner.lock();
        let plugin = self.inner.as_ptr();
        let uri = uri.inner.as_ptr();

        unsafe { lib::lilv_plugin_has_extension_data(plugin, uri) }
    }

    /// Get a sequence of all extension data provided by a plugin.
    #[must_use]
    pub fn extension_data(&self) -> Option<Nodes> {
        let _life = self.life.inner.lock();
        let plugin = self.inner.as_ptr();

        Some({
            let inner = unsafe { lib::lilv_plugin_get_extension_data(plugin) };
            let world = self.life.clone();
            Nodes { inner, life: world }
        })
    }

    /// Returns the number of ports.
    #[must_use]
    pub fn ports_count(&self) -> usize {
        let _life = self.life.inner.lock();
        let plugin = self.inner.as_ptr();
        unsafe { lib::lilv_plugin_get_num_ports(plugin) as _ }
    }

    /// Return the ranges for all ports.
    #[must_use]
    pub fn port_ranges_float(&self) -> Vec<FloatRanges> {
        let ports_count = self.ports_count();
        let mut min = vec![0_f32; ports_count];
        let mut max = vec![0_f32; ports_count];
        let mut default = vec![0_f32; ports_count];
        let plugin = self.inner.as_ptr();

        unsafe {
            let _life = self.life.inner.lock();
            lib::lilv_plugin_get_port_ranges_float(
                plugin,
                min.as_mut_ptr(),
                max.as_mut_ptr(),
                default.as_mut_ptr(),
            );
        }
        (0..ports_count)
            .map(|i| FloatRanges {
                min: min[i],
                max: max[i],
                default: default[i],
            })
            .collect()
    }

    /// Returns the number of ports that match all the given classes.
    #[must_use]
    pub fn num_ports_of_class(&self, classes: &[&Node]) -> usize {
        (0..self.ports_count())
            .filter_map(|index| self.port_by_index(index))
            .filter(|port| classes.iter().all(|cls| port.is_a(cls)))
            .count()
    }

    /// Returns wether or not the latency port can be found.
    #[must_use]
    pub fn has_latency(&self) -> bool {
        let _life = self.life.inner.lock();
        let plugin = self.inner.as_ptr();
        unsafe { lib::lilv_plugin_has_latency(plugin) }
    }

    /// Return the index of the plugin's latency port or `None` if it does not exist.
    #[must_use]
    pub fn latency_port_index(&self) -> Option<usize> {
        if self.has_latency() {
            let _life = self.life.inner.lock();
            let plugin = self.inner.as_ptr();
            Some(unsafe { lib::lilv_plugin_get_latency_port_index(plugin) as _ })
        } else {
            None
        }
    }

    /// Iterate through all the ports.
    pub fn iter_ports(&self) -> impl Iterator<Item = Port> {
        PortsIter {
            plugin: self.clone(),
            index: 0,
        }
    }

    /// Return the port by index or `None` if it does not exist.
    #[must_use]
    pub fn port_by_index(&self, index: usize) -> Option<Port> {
        let _life = self.life.inner.lock();
        let plugin = self.inner.as_ptr();
        let index = u32::try_from(index).ok()?;

        Some({
            let inner = NonNull::new(unsafe {
                lib::lilv_plugin_get_port_by_index(plugin, index as _) as _
            })?;
            Port {
                inner,
                plugin: self.clone(),
            }
        })
    }

    //// Get the port by the symbol.
    ///
    ///  Note: This function is slower than `port_by_index`, especially on
    ///  plugins with a very large number of ports.
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
                plugin: self.clone(),
            }
        })
    }

    /// Get a port on plugin by its lv2:designation.

    /// The designation of a port describes the meaning, assignment, allocation
    /// or role of the port, e.g. "left channel" or "gain". If found, the port
    /// with matching `port_class` and designation is be returned, otherwise
    /// `None` is returned. The `port_class` can be used to distinguish the
    /// input and output ports for a particular designation. If `port_class` is
    /// `None`, any port with the given designation will be returned.
    #[must_use]
    pub fn port_by_designation(
        &self,
        port_class: Option<&Node>,
        designation: &Node,
    ) -> Option<Port> {
        let _life = self.life.inner.lock();
        let plugin = self.inner.as_ptr();
        let port_class = port_class.map_or(std::ptr::null(), |n| n.inner.as_ptr());
        let designation = designation.inner.as_ptr();

        Some({
            let inner = NonNull::new(unsafe {
                lib::lilv_plugin_get_port_by_designation(plugin, port_class, designation) as _
            })?;
            Port {
                inner,
                plugin: self.clone(),
            }
        })
    }

    /// Get the project the plugin is a part of.
    ///
    /// More information about the project can be read with `World::find_nodes`.
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

    /// Returns the author name if present.
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

    /// Returns the author email if present.
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
        let ptr = NonNull::new(unsafe { lib::lilv_plugin_get_author_homepage(plugin) })?;
        let world = self.life.clone();

        Some(Node {
            inner: ptr,
            borrowed: false,
            life: world,
        })
    }

    /// `true` if the plugin has been replaced by another plugin.
    ///
    /// The plugin will still be usable, but hosts should hide them from their
    /// user interfaces to prevent users fromusing deprecated plugins.
    #[must_use]
    pub fn is_replaced(&self) -> bool {
        let _life = self.life.inner.lock();
        let plugin = self.inner.as_ptr();
        unsafe { lib::lilv_plugin_is_replaced(plugin) }
    }

    /// Get the resources related to plugin with lv2:appliesTo.
    ///
    /// Some plugin-related resources are not linked directly to the plugin with
    /// rdfs:seeAlso and thus will not be automatically loaded along with the
    /// plugin data (usually for performance reasons). All such resources of the
    /// given type related to plugin can be accessed with this function.
    ///
    /// If typ is `None`, all such resources will be returned, regardless of
    /// type.
    ///
    /// To actually load the data for each returned resource, use
    /// `World::load_resource()`.
    #[must_use]
    pub fn related(&self, typ: Option<&Node>) -> Option<Nodes> {
        let _life = self.life.inner.lock();
        let plugin = self.inner.as_ptr();
        let plugin_type = typ.map_or(std::ptr::null(), |n| n.inner.as_ptr() as _);

        Some({
            let inner = unsafe { lib::lilv_plugin_get_related(plugin, plugin_type) };
            let world = self.life.clone();
            Nodes { inner, life: world }
        })
    }

    /// Get all UIs for plugin.
    #[must_use]
    pub fn uis(&self) -> Option<Uis> {
        let _life = self.life.inner.lock();
        let plugin = self.inner.as_ptr();

        Some(Uis {
            inner: NonNull::new(unsafe { lib::lilv_plugin_get_uis(plugin) })?,
            life: self.life.clone(),
            plugin: self.clone(),
        })
    }

    /// Instantiate a plugin.
    ///
    /// # Safety
    /// Instantiating a plugin calls the plugin's code which itself may be
    /// unsafe.
    #[must_use]
    pub unsafe fn instantiate(
        &self,
        sample_rate: f64,
        features: &[lv2_raw::LV2Feature],
    ) -> Option<Instance> {
        let _life = self.life.inner.lock();
        let plugin = self.inner.as_ptr();
        let features: Vec<*const lv2_raw::LV2Feature> = features
            .iter()
            .map(|f| f as *const _)
            .chain(std::iter::once(std::ptr::null()))
            .collect();
        let inner = NonNull::new(lib::lilv_plugin_instantiate(
            plugin,
            sample_rate,
            features.as_ptr(),
        ))?;

        Some(Instance { inner })
    }
}

pub struct Plugins {
    pub(crate) life: Arc<Life>,
    pub(crate) ptr: *const lib::LilvPlugins,
}

impl Plugins {
    /// An iterable over all the plugins in the world.
    pub fn iter(&self) -> impl '_ + Iterator<Item = Plugin> {
        let _life = self.life.inner.lock();
        PluginsIter {
            plugins: self,
            iter: { unsafe { lib::lilv_plugins_begin(self.ptr) } },
        }
    }

    /// Get a plugin by its unique identifier.
    #[must_use]
    pub fn plugin(&self, uri: &Node) -> Option<Plugin> {
        let _life = self.life.inner.lock();
        let uri_ptr = uri.inner.as_ptr();
        let plugin_ptr: *mut lib::LilvPlugin =
            unsafe { lib::lilv_plugins_get_by_uri(self.ptr, uri_ptr) as *mut _ };
        Some(Plugin {
            life: self.life.clone(),
            inner: NonNull::new(plugin_ptr)?,
        })
    }

    /// The number of plugins loaded.
    #[must_use]
    pub fn count(&self) -> usize {
        let _life = self.life.inner.lock();
        let size = unsafe { lib::lilv_plugins_size(self.ptr) };
        size as usize
    }
}

impl IntoIterator for Plugins {
    type Item = Plugin;

    type IntoIter = PluginsIter<Plugins>;

    fn into_iter(self) -> Self::IntoIter {
        let iter = {
            let _life = self.life.inner.lock();
            unsafe { lib::lilv_plugins_begin(self.ptr) }
        };
        PluginsIter {
            plugins: self,
            iter,
        }
    }
}

/// An iterator over plugins.
pub struct PluginsIter<PS> {
    pub(crate) plugins: PS,
    pub(crate) iter: *mut lib::LilvIter,
}

impl<PS> Iterator for PluginsIter<PS>
where
    PS: Borrow<Plugins>,
{
    type Item = Plugin;

    fn next(&mut self) -> Option<Plugin> {
        let _life = self.plugins.borrow().life.inner.lock();
        let ptr: *mut lib::LilvPlugin =
            unsafe { lib::lilv_plugins_get(self.plugins.borrow().ptr, self.iter) } as *mut _;
        self.iter = unsafe { lib::lilv_plugins_next(self.plugins.borrow().ptr, self.iter) };
        match NonNull::new(ptr) {
            Some(ptr) => Some(Plugin {
                life: self.plugins.borrow().life.clone(),
                inner: ptr,
            }),
            None => None,
        }
    }
}

/// Can be used to instantiave LV2 plugins.
struct PortsIter {
    pub(crate) plugin: Plugin,
    pub(crate) index: usize,
}

impl Iterator for PortsIter {
    type Item = Port;

    fn next(&mut self) -> Option<Port> {
        let index = self.index;
        self.index += 1;
        self.plugin.port_by_index(index)
    }
}

unsafe impl Send for PluginClass {}
unsafe impl Sync for PluginClass {}

pub struct PluginClass {
    pub(crate) inner: NonNull<lib::LilvPluginClass>,
    life: Arc<Life>,
}

impl PluginClass {
    pub(crate) fn new_borrowed(ptr: NonNull<lib::LilvPluginClass>, world: Arc<Life>) -> Self {
        Self {
            inner: ptr,
            life: world,
        }
    }

    #[must_use]
    pub fn parent_uri(&self) -> Option<Node> {
        let _life = self.life.inner.lock();
        let inner = self.inner.as_ptr();

        Some({
            let ptr = NonNull::new(unsafe { lib::lilv_plugin_class_get_parent_uri(inner) as _ })?;
            let world = self.life.clone();
            Node {
                inner: ptr,
                borrowed: true,
                life: world,
            }
        })
    }

    #[must_use]
    pub fn uri(&self) -> Option<Node> {
        let _life = self.life.inner.lock();
        let inner = self.inner.as_ptr();

        {
            let ptr = NonNull::new(unsafe { lib::lilv_plugin_class_get_uri(inner) as _ })?;
            let world = self.life.clone();
            Node {
                inner: ptr,
                borrowed: true,
                life: world,
            }
        }
        .into()
    }

    /// # Panics
    /// Panics if the label could not be obtained.
    #[must_use]
    pub fn label(&self) -> Node {
        let _life = self.life.inner.lock();
        let inner = self.inner.as_ptr();

        {
            let ptr =
                NonNull::new(unsafe { lib::lilv_plugin_class_get_label(inner) as _ }).unwrap();
            let world = self.life.clone();
            Node {
                inner: ptr,
                borrowed: true,
                life: world,
            }
        }
    }

    #[must_use]
    pub fn children(&self) -> Option<PluginClasses> {
        let _life = self.life.inner.lock();
        let inner = self.inner.as_ptr();
        PluginClasses {
            inner: NonNull::new(unsafe { lib::lilv_plugin_class_get_children(inner) })?,
            life: self.life.clone(),
        }
        .into()
    }
}

pub struct PluginClasses {
    pub(crate) inner: NonNull<lib::LilvPluginClasses>,
    pub(crate) life: Arc<Life>,
}

impl PluginClasses {
    #[must_use]
    pub fn iter(&self) -> PluginClassesIter {
        let _life = self.life.inner.lock();
        PluginClassesIter {
            classes: self.inner.as_ptr(),
            iter: unsafe { lib::lilv_plugin_classes_begin(self.inner.as_ptr()) },
            life: self.life.clone(),
        }
    }

    #[must_use]
    pub fn count(&self) -> usize {
        let _life = self.life.inner.lock();
        unsafe { lib::lilv_plugin_classes_size(self.inner.as_ptr()) as _ }
    }

    #[must_use]
    pub fn get_by_uri(&self, uri: &Node) -> Option<PluginClass> {
        let _life = self.life.inner.lock();
        let inner = self.inner.as_ptr();
        let uri = uri.inner.as_ptr();

        Some(PluginClass::new_borrowed(
            NonNull::new(unsafe { lib::lilv_plugin_classes_get_by_uri(inner, uri) as _ })?,
            self.life.clone(),
        ))
    }
}

pub struct PluginClassesIter {
    classes: *mut lib::LilvPluginClasses,
    iter: *mut lib::LilvIter,
    life: Arc<Life>,
}

impl Iterator for PluginClassesIter {
    type Item = PluginClass;

    #[must_use]
    fn next(&mut self) -> Option<PluginClass> {
        let _life = self.life.inner.lock();
        let ptr = unsafe { lib::lilv_plugin_classes_get(self.classes, self.iter) };
        if ptr.is_null() {
            None
        } else {
            self.iter = unsafe { lib::lilv_plugin_classes_next(self.classes, self.iter) };
            Some(PluginClass::new_borrowed(
                NonNull::new(ptr as _)?,
                self.life.clone(),
            ))
        }
    }
}
