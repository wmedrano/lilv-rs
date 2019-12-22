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

    pub fn load_all(&self) {
        unsafe { lib::lilv_world_load_all(self.inner.inner.write().as_ptr()) }
    }

    pub fn load_bundle(&self, bundle_uri: &Node) {
        let world = self.inner.inner.write().as_ptr();
        let bundle_uri = bundle_uri.inner.read().as_ptr();

        unsafe { lib::lilv_world_load_bundle(world, bundle_uri) }
    }

    pub fn load_specifications(&self) {
        let world = self.inner.inner.write().as_ptr();
        unsafe { lib::lilv_world_load_specifications(world) }
    }

    pub fn load_plugin_classes(&self) {
        let world = self.inner.inner.write().as_ptr();
        unsafe { lib::lilv_world_load_plugin_classes(world) }
    }

    pub fn unload_bundle(&self, bundle_uri: &Node) -> bool {
        let world = self.inner.inner.write().as_ptr();
        let bundle_uri = bundle_uri.inner.read().as_ptr();

        unsafe { lib::lilv_world_unload_bundle(world, bundle_uri) == 0 }
    }

    pub fn load_resource(&self, resource: &Node) -> Result<usize, ()> {
        let world = self.inner.inner.write().as_ptr();
        let resource = resource.inner.read().as_ptr();

        match unsafe { lib::lilv_world_load_resource(world, resource) } {
            -1 => Err(()),
            n => Ok(n as _),
        }
    }

    pub fn unload_resource(&self, resource: &Node) -> bool {
        let world = self.inner.inner.write().as_ptr();
        let resource = resource.inner.read().as_ptr();

        unsafe { lib::lilv_world_unload_resource(world, resource) == 0 }
    }

    pub fn plugin_class(&self) -> PluginClass {
        let world = self.inner.inner.read().as_ptr();

        PluginClass::new_borrowed(
            NonNull::new(unsafe { lib::lilv_world_get_plugin_class(world) as _ }).unwrap(),
            self.inner.clone(),
        )
    }

    pub fn plugin_classes(&self) -> PluginClasses {
        let world = self.inner.inner.read().as_ptr();

        PluginClasses::new_borrowed(
            NonNull::new(unsafe { lib::lilv_world_get_plugin_classes(world) as _ }).unwrap(),
            self.inner.clone(),
        )
    }

    pub fn all_plugins(&self) -> Plugins {
        let world = self.inner.inner.read().as_ptr();

        Plugins::new_borrowed(
            NonNull::new(unsafe { lib::lilv_world_get_all_plugins(world) as _ }).unwrap(),
            self.inner.clone(),
        )
    }

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

    pub fn symbol(&self, subject: &Node) -> Option<Node> {
        let world = self.inner.inner.read().as_ptr();
        let subject = subject.inner.read().as_ptr();

        Some(Node::new(
            NonNull::new(unsafe { lib::lilv_world_get_symbol(world, subject) })?,
            self.inner.clone(),
        ))
    }
}

impl Drop for InnerWorld {
    fn drop(&mut self) {
        unsafe { lib::lilv_world_free(self.inner.write().as_ptr()) }
    }
}
