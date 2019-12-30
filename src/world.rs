use crate::node::Node;
use crate::nodes::Nodes;
use crate::plugin_class::PluginClass;
use crate::plugin_classes::PluginClasses;
use crate::plugins::Plugins;
use lilv_sys as lib;
use parking_lot::RwLock;
use std::ptr::NonNull;
use std::sync::Arc;

unsafe impl Send for InnerWorld {}
unsafe impl Sync for InnerWorld {}

/// The world represents all Lilv state. It is used to discover/load/cache LV2
/// data (plugins, UIs, and extensions).
pub struct World {
    inner: Arc<InnerWorld>,
}

#[doc(hidden)]
pub struct InnerWorld {
    pub(crate) inner: RwLock<NonNull<lib::LilvWorldImpl>>,
}

impl World {
    /// Initializes a new, empty world.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(InnerWorld {
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
    pub fn new_file_uri(&self, host: Option<&str>, path: &str) -> Node {
        let world = self.inner.inner.write().as_ptr();

        let host_c;
        let host = if let Some(host) = host {
            host_c = crate::make_c_string(host);
            crate::choose_string(host, &host_c)
        } else {
            host_c = None;
            let _ = &host_c;
            std::ptr::null()
        };

        let path_c = crate::make_c_string(path);
        let path = crate::choose_string(path, &path_c);

        Node::new(
            NonNull::new(unsafe { lib::lilv_new_file_uri(world, host, path) }).unwrap(),
            self.inner.clone(),
        )
    }

    /// Creates a new string value (with no language).
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
    pub fn new_int(&self, value: i32) -> Node {
        let world = self.inner.inner.write().as_ptr();

        Node::new(
            NonNull::new(unsafe { lib::lilv_new_int(world, value) }).unwrap(),
            self.inner.clone(),
        )
    }

    /// Creates a new floating point value.
    pub fn new_float(&self, value: f32) -> Node {
        let world = self.inner.inner.write().as_ptr();

        Node::new(
            NonNull::new(unsafe { lib::lilv_new_float(world, value) }).unwrap(),
            self.inner.clone(),
        )
    }

    /// Creates a new boolean value.
    pub fn new_bool(&self, value: bool) -> Node {
        let world = self.inner.inner.write().as_ptr();

        Node::new(
            NonNull::new(unsafe { lib::lilv_new_bool(world, value) }).unwrap(),
            self.inner.clone(),
        )
    }

    /// Load all installed LV2 bundles on the system.
    ///
    /// # Example
    /// ```
    /// let world = lilv::World::new();
    /// world.load_all();
    /// ```
    pub fn load_all(&self) {
        unsafe { lib::lilv_world_load_all(self.inner.inner.write().as_ptr()) }
    }

    /// Load a specific bundle. `bundle_uri` must be a fully qualified URI to
    /// the bundle directory, with the trailing slash, eg `file:///usr/lib/lv2/foo.lv2/`.
    pub fn load_bundle(&self, bundle_uri: &Node) {
        let world = self.inner.inner.write().as_ptr();
        let bundle_uri = bundle_uri.inner.read().as_ptr();

        unsafe { lib::lilv_world_load_bundle(world, bundle_uri) }
    }

    /// Load all specifications from currently loaded bundles
    ///
    /// This is for hosts tha explicitly load specific bundles, its use is not
    /// necessary when using `World::load_all`. This function parses the specifications and adsds them to the model.
    pub fn load_specifications(&self) {
        let world = self.inner.inner.write().as_ptr();
        unsafe { lib::lilv_world_load_specifications(world) }
    }

    /// Load all plugin classes from currently loaded specifications.
    ///
    /// Must be called after `World::load_specifications`. This is for hosts
    /// that explicitly load specific bundles; its use is not necessary when
    /// using `World::load_all`.
    pub fn load_plugin_classes(&self) {
        let world = self.inner.inner.write().as_ptr();
        unsafe { lib::lilv_world_load_plugin_classes(world) }
    }

    /// Unload a specific bundle.
    ///
    /// This unloads statements loaded by `load_bundle`. Note this is not
    /// necessarily all information loaded from the bundle. If any  resources
    /// have been separately loaded with `World::load_resource`, they must be
    /// separately unloaded with `World::unload_resource`.
    pub fn unload_bundle(&self, bundle_uri: &Node) -> bool {
        let world = self.inner.inner.write().as_ptr();
        let bundle_uri = bundle_uri.inner.read().as_ptr();

        unsafe { lib::lilv_world_unload_bundle(world, bundle_uri) == 0 }
    }

    /// Load all the data associated with the given resource.
    ///
    /// # Return
    /// The number of files parsed.
    pub fn load_resource(&self, resource: &Node) -> Result<usize, ()> {
        let world = self.inner.inner.write().as_ptr();
        let resource = resource.inner.read().as_ptr();

        match unsafe { lib::lilv_world_load_resource(world, resource) } {
            -1 => Err(()),
            n => Ok(n as _),
        }
    }

    /// Unload all the data associated with the given resource.
    pub fn unload_resource(&self, resource: &Node) -> bool {
        let world = self.inner.inner.write().as_ptr();
        let resource = resource.inner.read().as_ptr();

        unsafe { lib::lilv_world_unload_resource(world, resource) == 0 }
    }

    /// Get the parent of all other plugin classes, lv2:Plugin.
    pub fn plugin_class(&self) -> PluginClass {
        let world = self.inner.inner.read().as_ptr();

        PluginClass::new_borrowed(
            NonNull::new(unsafe { lib::lilv_world_get_plugin_class(world) as _ }).unwrap(),
            self.inner.clone(),
        )
    }

    /// Return all found plugin classes.
    pub fn plugin_classes(&self) -> PluginClasses {
        let world = self.inner.inner.read().as_ptr();

        PluginClasses::new_borrowed(
            NonNull::new(unsafe { lib::lilv_world_get_plugin_classes(world) as _ }).unwrap(),
            self.inner.clone(),
        )
    }

    /// Return all plugins.
    pub fn all_plugins(&self) -> Plugins {
        let world = self.inner.inner.read().as_ptr();

        Plugins::new_borrowed(
            NonNull::new(unsafe { lib::lilv_world_get_all_plugins(world) as _ }).unwrap(),
            self.inner.clone(),
        )
    }

    /// Find nodes matching a triple pattern. Either subject or object may be
    /// `None`, but not both.
    pub fn find_nodes(
        &self,
        subject: Option<&Node>,
        predicate: &Node,
        object: Option<&Node>,
    ) -> Option<Nodes> {
        let world = self.inner.inner.read().as_ptr();
        let subject = subject
            .map(|n| n.inner.read().as_ptr() as _)
            .unwrap_or(std::ptr::null());
        let predicate = predicate.inner.read().as_ptr();
        let object = object
            .map(|n| n.inner.read().as_ptr() as _)
            .unwrap_or(std::ptr::null());

        Some(Nodes::new(
            NonNull::new(unsafe { lib::lilv_world_find_nodes(world, subject, predicate, object) })?,
            self.inner.clone(),
        ))
    }

    /// Find a single node that matches a pattern. Exactly one of `subject`,
    /// `predicate`, or `object` must be `None`.
    pub fn get(
        &self,
        subject: Option<&Node>,
        predicate: Option<&Node>,
        object: Option<&Node>,
    ) -> Option<Node> {
        let world = self.inner.inner.read().as_ptr();
        let subject = subject
            .map(|n| n.inner.read().as_ptr() as _)
            .unwrap_or(std::ptr::null());
        let predicate = predicate
            .map(|n| n.inner.read().as_ptr() as _)
            .unwrap_or(std::ptr::null());
        let object = object
            .map(|n| n.inner.read().as_ptr() as _)
            .unwrap_or(std::ptr::null());

        Some(Node::new(
            NonNull::new(unsafe { lib::lilv_world_get(world, subject, predicate, object) })?,
            self.inner.clone(),
        ))
    }

    /// Returns true iff a statement matching a certain pattern exists.
    pub fn ask(
        &self,
        subject: Option<&Node>,
        predicate: Option<&Node>,
        object: Option<&Node>,
    ) -> bool {
        let world = self.inner.inner.read().as_ptr();
        let subject = subject
            .map(|n| n.inner.read().as_ptr() as _)
            .unwrap_or(std::ptr::null());
        let predicate = predicate
            .map(|n| n.inner.read().as_ptr() as _)
            .unwrap_or(std::ptr::null());
        let object = object
            .map(|n| n.inner.read().as_ptr() as _)
            .unwrap_or(std::ptr::null());

        unsafe { lib::lilv_world_ask(world, subject, predicate, object) }
    }

    /// Get an LV2 symbol for some subject.
    ///
    /// This will return the lv2:symbol property of the subject if it is given
    /// explicitly. Otherwise it will attempt to derive a symbol from the URI.
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

impl Drop for InnerWorld {
    fn drop(&mut self) {
        unsafe { lib::lilv_world_free(self.inner.write().as_ptr()) }
    }
}
