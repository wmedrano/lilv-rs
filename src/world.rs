use crate::node::{Node, Nodes};
use crate::plugin::PluginsIter;
use crate::plugin_class::PluginClass;
use crate::Plugin;
use lilv_sys as lib;
use parking_lot::RwLock;
use std::ptr::NonNull;
use std::sync::Arc;

unsafe impl Send for Life {}
unsafe impl Sync for Life {}

/// The world represents all Lilv state. It is used to discover/load/cache LV2
/// data (plugins, UIs, and extensions).
pub struct World {
    inner: Arc<Life>,
}

#[doc(hidden)]
pub struct Life {
    pub(crate) inner: RwLock<NonNull<lib::LilvWorldImpl>>,
}

impl World {
    /// Initializes a new, empty world.
    ///
    /// # Panics
    /// Panics if the world could not be created.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Life {
                inner: RwLock::new(NonNull::new(unsafe { lib::lilv_world_new() }).unwrap()),
            }),
        }
    }

    /// Sets an option for the world.
    pub fn set_option(&self, uri: &str, value: &Node) {
        let world = self.inner.inner.write().as_ptr();
        let uri_c = crate::make_c_string(uri);
        let uri = crate::choose_string(uri, &uri_c);
        let value = value.inner.read().as_ptr();

        unsafe { lib::lilv_world_set_option(world, uri, value) }
    }

    /// Creates a new URI value.
    ///
    /// # Panics
    /// Panics on failure.
    #[must_use]
    pub fn new_uri(&self, uri: &str) -> Node {
        let world = self.inner.inner.write().as_ptr();
        let uri_c = crate::make_c_string(uri);
        let uri = crate::choose_string(uri, &uri_c);

        Node::new(
            NonNull::new(unsafe { lib::lilv_new_uri(world, uri) }).unwrap(),
            self.inner.clone(),
        )
    }

    /// Creates a new file URI value.
    ///
    /// # Panics
    /// Panics on failure.
    #[must_use]
    pub fn new_file_uri(&self, host: Option<&str>, path: &str) -> Node {
        let world = self.inner.inner.write().as_ptr();

        let host = host
            .iter()
            .find_map(|h| std::ffi::CString::new(h.as_bytes()).ok());
        let path = std::ffi::CString::new(path.as_bytes()).unwrap();

        let host_ptr = host.map_or(std::ptr::null(), |h| h.as_ptr());
        let path_ptr = path.as_ptr();

        Node::new(
            NonNull::new(unsafe { lib::lilv_new_file_uri(world, host_ptr, path_ptr) }).unwrap(),
            self.inner.clone(),
        )
    }

    /// Creates a new string value (with no language).
    ///
    /// # Panics
    /// Panics on failure.
    #[must_use]
    pub fn new_string(&self, string: &str) -> Node {
        let world = self.inner.inner.write().as_ptr();
        let string_c = crate::make_c_string(string);
        let string = crate::choose_string(string, &string_c);

        Node::new(
            NonNull::new(unsafe { lib::lilv_new_string(world, string) }).unwrap(),
            self.inner.clone(),
        )
    }

    /// Creates a new integer value.
    ///
    /// # Panics
    /// Panics on failure.
    #[must_use]
    pub fn new_int(&self, value: i32) -> Node {
        let world = self.inner.inner.write().as_ptr();

        Node::new(
            NonNull::new(unsafe { lib::lilv_new_int(world, value) }).unwrap(),
            self.inner.clone(),
        )
    }

    /// Creates a new floating point value.
    ///
    /// # Panics
    /// Panics on failure.
    #[must_use]
    pub fn new_float(&self, value: f32) -> Node {
        let world = self.inner.inner.write().as_ptr();

        Node::new(
            NonNull::new(unsafe { lib::lilv_new_float(world, value) }).unwrap(),
            self.inner.clone(),
        )
    }

    /// Creates a new boolean value.
    ///
    /// # Panics
    /// Panics on failure.
    #[must_use]
    pub fn new_bool(&self, value: bool) -> Node {
        let world = self.inner.inner.write().as_ptr();

        Node::new(
            NonNull::new(unsafe { lib::lilv_new_bool(world, value) }).unwrap(),
            self.inner.clone(),
        )
    }

    /// Loads all installed LV2 bundles on the system.
    ///
    /// # Example
    /// ```
    /// let world = lilv::World::new();
    /// world.load_all();
    /// ```
    pub fn load_all(&self) {
        unsafe { lib::lilv_world_load_all(self.inner.inner.write().as_ptr()) }
    }

    /// Loads a specific bundle. `bundle_uri` must be a fully qualified URI to the bundle directory,
    /// with the trailing slash, eg `file:///usr/lib/lv2/foo.lv2/`.
    pub fn load_bundle(&self, bundle_uri: &Node) {
        let world = self.inner.inner.write().as_ptr();
        let bundle_uri = bundle_uri.inner.read().as_ptr();

        unsafe { lib::lilv_world_load_bundle(world, bundle_uri) }
    }

    /// Loads all specifications from currently loaded bundles.
    ///
    /// This is for hosts that explicitly load specific bundles, its use is not
    /// necessary when using [`load_all`](#method.load_all). This function parses the specifications
    /// and adds them to the model.
    pub fn load_specifications(&self) {
        let world = self.inner.inner.write().as_ptr();
        unsafe { lib::lilv_world_load_specifications(world) }
    }

    /// Load all plugin classes from currently loaded specifications.
    ///
    /// Must be called after [`load_specifications`](#method.load_specifications). This is for hosts
    /// that explicitly load specific bundles; its use is not necessary when using
    /// [`load_all`](#method.load_all).
    pub fn load_plugin_classes(&self) {
        let world = self.inner.inner.write().as_ptr();
        unsafe { lib::lilv_world_load_plugin_classes(world) }
    }

    /// Unload a specific bundle.
    ///
    /// This unloads statements loaded by `load_bundle`. Note this is not necessarily all
    /// information loaded from the bundle. If any  resources have been separately loaded with
    /// [`load_resource`](#method.load_resource), they must be separately unloaded with
    /// [`unload_resource`](#method.unload_resource).
    ///
    /// # Safety
    /// Unloading bundles that are in use by the host will cause undefined
    /// behaviour.
    pub unsafe fn unload_bundle(&self, bundle_uri: &Node) -> bool {
        let world = self.inner.inner.write().as_ptr();
        let bundle_uri = bundle_uri.inner.read().as_ptr();

        lib::lilv_world_unload_bundle(world, bundle_uri) == 0
    }

    /// Load all the data associated with the given resource.
    ///
    /// # Return
    /// The number of files parsed.
    #[allow(clippy::cast_sign_loss)]
    pub fn load_resource(&self, resource: &Node) -> Option<usize> {
        let world = self.inner.inner.write().as_ptr();
        let resource = resource.inner.read().as_ptr();

        match unsafe { lib::lilv_world_load_resource(world, resource) } {
            -1 => None,
            n => Some(n as usize),
        }
    }

    /// Unload all the data associated with the given resource.
    ///
    /// # Safety
    /// Unloading resources that are in use by the host will cause undefined
    /// behaviour.
    pub unsafe fn unload_resource(&self, resource: &Node) -> bool {
        let world = self.inner.inner.write().as_ptr();
        let resource = resource.inner.read().as_ptr();

        lib::lilv_world_unload_resource(world, resource) == 0
    }

    /// Get the parent of all other plugin classes, lv2:Plugin.
    #[must_use]
    pub fn plugin_class(&self) -> Option<PluginClass> {
        let world = self.inner.inner.read().as_ptr();

        Some(PluginClass::new_borrowed(
            NonNull::new(unsafe { lib::lilv_world_get_plugin_class(world) as _ })?,
            self.inner.clone(),
        ))
    }

    /// An iterable over all the plugins in the world.
    #[must_use]
    pub fn plugins(&self) -> PluginsIter {
        let (ptr, iter) = {
            let world = self.inner.inner.read();
            let ptr = unsafe { lib::lilv_world_get_all_plugins(world.as_ptr()) };
            let iter = unsafe { lib::lilv_plugins_begin(ptr) };
            (ptr, iter)
        };

        PluginsIter {
            world: self.inner.clone(),
            ptr,
            iter,
        }
    }

    /// Get a plugin by its unique identifier.
    pub fn plugin(&self, uri: &Node) -> Option<Plugin> {
        let plugin_ptr: *mut lib::LilvPlugin = {
            let world = self.inner.inner.read();
            let plugins_ptr = unsafe { lib::lilv_world_get_all_plugins(world.as_ptr()) };
            let uri_ptr = uri.inner.read().as_ptr();
            unsafe { lib::lilv_plugins_get_by_uri(plugins_ptr, uri_ptr) }
        } as _;
        Some(Plugin {
            world: self.inner.clone(),
            inner: RwLock::new(NonNull::new(plugin_ptr)?),
        })
    }

    /// The number of plugins loaded.
    #[must_use]
    pub fn plugins_count(&self) -> usize {
        let world = self.inner.inner.read();
        let ptr = unsafe { lib::lilv_world_get_all_plugins(world.as_ptr()) };
        let size = unsafe { lib::lilv_plugins_size(ptr) };
        size as usize
    }

    /// Find nodes matching a triple pattern. Either subject or object may be `None`, but not both.
    pub fn find_nodes(
        &self,
        subject: Option<&Node>,
        predicate: &Node,
        object: Option<&Node>,
    ) -> Option<Nodes> {
        let world = self.inner.inner.read().as_ptr();
        let subject = subject.map_or(std::ptr::null(), |n| n.inner.read().as_ptr() as _);
        let predicate = predicate.inner.read().as_ptr();
        let object = object.map_or(std::ptr::null(), |n| n.inner.read().as_ptr() as _);

        Some(Nodes::new(
            NonNull::new(unsafe { lib::lilv_world_find_nodes(world, subject, predicate, object) })?,
            self.inner.clone(),
        ))
    }

    /// Find a single node that matches a pattern. Exactly one of `subject`, `predicate`, or
    /// `object` must be `None`.
    #[must_use]
    pub fn get(
        &self,
        subject: Option<&Node>,
        predicate: Option<&Node>,
        object: Option<&Node>,
    ) -> Option<Node> {
        let world = self.inner.inner.read().as_ptr();
        let subject = subject.map_or(std::ptr::null(), |n| n.inner.read().as_ptr() as _);
        let predicate = predicate.map_or(std::ptr::null(), |n| n.inner.read().as_ptr() as _);
        let object = object.map_or(std::ptr::null(), |n| n.inner.read().as_ptr() as _);

        Some(Node::new(
            NonNull::new(unsafe { lib::lilv_world_get(world, subject, predicate, object) })?,
            self.inner.clone(),
        ))
    }

    /// Returns true iff a statement matching a certain pattern exists.
    #[must_use]
    pub fn ask(
        &self,
        subject: Option<&Node>,
        predicate: Option<&Node>,
        object: Option<&Node>,
    ) -> bool {
        let world = self.inner.inner.read().as_ptr();
        let subject = subject.map_or(std::ptr::null(), |n| n.inner.read().as_ptr() as _);
        let predicate = predicate.map_or(std::ptr::null(), |n| n.inner.read().as_ptr() as _);
        let object = object.map_or(std::ptr::null(), |n| n.inner.read().as_ptr() as _);

        unsafe { lib::lilv_world_ask(world, subject, predicate, object) }
    }

    /// Get an LV2 symbol for some subject.
    ///
    /// This will return the lv2:symbol property of the subject if it is given explicitly. Otherwise
    /// it will attempt to derive a symbol from the URI.
    pub fn symbol(&self, subject: &Node) -> Option<Node> {
        let world = self.inner.inner.read().as_ptr();
        let subject = subject.inner.read().as_ptr();

        Some(Node::new(
            NonNull::new(unsafe { lib::lilv_world_get_symbol(world, subject) })?,
            self.inner.clone(),
        ))
    }
}

impl Default for World {
    /// Return a new empty world.
    fn default() -> World {
        World::new()
    }
}

impl Drop for Life {
    fn drop(&mut self) {
        unsafe { lib::lilv_world_free(self.inner.write().as_ptr()) }
    }
}
