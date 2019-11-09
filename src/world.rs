use crate::node::Node;
use crate::nodes::Nodes;
use crate::plugin_class::PluginClass;
use crate::plugin_classes::PluginClasses;
use crate::plugins::Plugins;
use lilv_sys::*;
use std::ffi::CStr;
use std::marker::PhantomData;
use std::ptr;
use std::rc::Rc;
use std::sync::RwLock;

pub struct World(pub(crate) RwLock<*mut LilvWorld>);

pub(crate) fn new_node<'a>(world: &Rc<World>, node: *mut LilvNode) -> Node<'a> {
    Node {
        node,
        world: world.clone(),
        owned: true,
        _phantom: PhantomData,
    }
}

pub(crate) fn ref_node<'a>(world: &Rc<World>, node: *const LilvNode) -> Node<'a> {
    Node {
        node: node as *mut LilvNode,
        world: world.clone(),
        owned: false,
        _phantom: PhantomData,
    }
}

impl World {
    /// Initialize a new, empty world.
    /// If initialization fails, `None` is returned.
    pub fn new() -> Option<Rc<World>> {
        let ptr = unsafe { lilv_world_new() };
        if ptr.is_null() {
            None
        } else {
            Some(Rc::new(World(RwLock::new(ptr))))
        }
    }

    pub fn with_load_all() -> Option<Rc<World>> {
        let world = World::new();
        if let Some(world) = &world {
            world.load_all();
        }
        world
    }
}

pub trait WorldImpl {
    fn set_option<'a>(&self, uri: &CStr, value: &Node<'a>);
    fn load_all(&self);
    fn load_bundle<'a>(&self, bundle_uri: &Node<'a>);
    fn load_specifications(&self);
    fn load_plugin_classes(&self);
    fn unload_bundle<'a>(&self, bundle_uri: &Node<'a>) -> Result<(), ()>;
    fn load_resource<'a>(&self, bundle_uri: &Node<'a>) -> Result<usize, ()>;
    fn unload_resource<'a>(&self, bundle_uri: &Node<'a>) -> Result<(), ()>;
    fn plugin_class(&self) -> PluginClass;
    fn plugin_classes(&self) -> PluginClasses;
    fn all_plugins(&self) -> Plugins;
    fn find_nodes<'a, S, P, O>(&self, subject: S, predicate: P, object: O) -> Option<Nodes>
    where
        S: Into<Option<&'a Node<'a>>>,
        P: Into<Option<&'a Node<'a>>>,
        O: Into<Option<&'a Node<'a>>>;
    fn get<'a, S, P, O>(&self, subject: S, predicate: P, object: O) -> Option<Node<'a>>
    where
        S: Into<Option<&'a Node<'a>>>,
        P: Into<Option<&'a Node<'a>>>,
        O: Into<Option<&'a Node<'a>>>;
    fn ask<'a, S, P, O>(&self, subject: S, predicate: P, object: O) -> bool
    where
        S: Into<Option<&'a Node<'a>>>,
        P: Into<Option<&'a Node<'a>>>,
        O: Into<Option<&'a Node<'a>>>;
    fn symbol<'a>(&self, subject: &Node<'a>) -> Option<Node>;
    fn new_uri<'a>(&self, uri: &CStr) -> Node<'a>;
    fn new_file_uri<'a>(&self, host: Option<&CStr>, path: &CStr) -> Node<'a>;
    fn new_string(&self, str: &CStr) -> Node;
    fn new_int(&self, value: i32) -> Node;
    fn new_float(&self, value: f32) -> Node;
    fn new_bool(&self, value: bool) -> Node;
}

impl WorldImpl for Rc<World> {
    /// Set an option for the world.
    ///
    /// Currently recognized options:
    /// * LILV_OPTION_FILTER_LANG
    /// * LILV_OPTION_DYN_MANIFEST
    fn set_option<'a>(&self, uri: &CStr, value: &Node<'a>) {
        unsafe { lilv_world_set_option(*self.0.write().unwrap(), uri.as_ptr(), value.node) }
    }

    /// Load all installed LV2 bundles on the system.
    /// This is the recommended way for hosts to load LV2 data.  It implements the
    /// established/standard best practice for discovering all LV2 data on the
    /// system.  The environment variable `LV2_PATH` may be used to control where
    /// this function will look for bundles.
    ///
    /// Hosts should use this function rather than explicitly load bundles, except
    /// in special circumstances (e.g. development utilities, or hosts that ship
    /// with special plugin bundles which are installed to a known location).
    fn load_all(&self) {
        unsafe { lilv_world_load_all(*self.0.write().unwrap()) }
    }

    fn load_bundle<'a>(&self, bundle_uri: &Node<'a>) {
        unsafe { lilv_world_load_bundle(*self.0.write().unwrap(), bundle_uri.node) }
    }

    fn load_specifications(&self) {
        unsafe { lilv_world_load_specifications(*self.0.write().unwrap()) }
    }

    fn load_plugin_classes(&self) {
        unsafe { lilv_world_load_plugin_classes(*self.0.write().unwrap()) }
    }

    fn unload_bundle<'a>(&self, bundle_uri: &Node<'a>) -> Result<(), ()> {
        if unsafe { lilv_world_unload_bundle(*self.0.write().unwrap(), bundle_uri.node) == 0 } {
            Ok(())
        } else {
            Err(())
        }
    }

    fn load_resource<'a>(&self, resource: &Node<'a>) -> Result<usize, ()> {
        unsafe {
            match lilv_world_load_resource(*self.0.write().unwrap(), resource.node) {
                -1 => Err(()),
                count => Ok(count as usize),
            }
        }
    }

    fn unload_resource<'a>(&self, resource: &Node<'a>) -> Result<(), ()> {
        if unsafe { lilv_world_unload_resource(*self.0.write().unwrap(), resource.node) == 0 } {
            Ok(())
        } else {
            Err(())
        }
    }

    fn plugin_class(&self) -> PluginClass {
        PluginClass {
            plugin_class: unsafe { lilv_world_get_plugin_class(*self.0.read().unwrap()) }
                as *mut LilvPluginClass,
            world: self.clone(),
        }
    }

    fn plugin_classes(&self) -> PluginClasses {
        let plugin_classes: *const LilvPluginClasses =
            unsafe { lilv_world_get_plugin_classes(*self.0.read().unwrap()) };
        PluginClasses {
            plugin_classes,
            owned: false,
            world: self.clone(),
        }
    }

    /// Return a list of all found plugins.
    /// The returned list contains just enough references to query
    /// or instantiate plugins.  The data for a particular plugin will not be
    /// loaded into memory until a call to a method on [`Plugin`](struct.Plugin.html) results in
    /// a query (at which time the data is cached with the [`Plugin`](struct.Plugin.html) so future
    /// queries are very fast).
    fn all_plugins(&self) -> Plugins {
        Plugins {
            plugins: unsafe { lilv_world_get_all_plugins(*self.0.write().unwrap()) },
            world: self.clone(),
        }
    }

    fn find_nodes<'a, S, P, O>(&self, subject: S, predicate: P, object: O) -> Option<Nodes>
    where
        S: Into<Option<&'a Node<'a>>>,
        P: Into<Option<&'a Node<'a>>>,
        O: Into<Option<&'a Node<'a>>>,
    {
        let subject = subject.into().map_or(ptr::null(), |x| x.node);
        let predicate = predicate.into().map_or(ptr::null(), |x| x.node);
        let object = object.into().map_or(ptr::null(), |x| x.node);
        let nodes =
            unsafe { lilv_world_find_nodes(*self.0.write().unwrap(), subject, predicate, object) };
        if nodes.is_null() {
            None
        } else {
            Some(Nodes {
                nodes,
                world: self.clone(),
                owned: true,
            })
        }
    }

    fn get<'a, S, P, O>(&self, subject: S, predicate: P, object: O) -> Option<Node<'a>>
    where
        S: Into<Option<&'a Node<'a>>>,
        P: Into<Option<&'a Node<'a>>>,
        O: Into<Option<&'a Node<'a>>>,
    {
        let subject = subject.into().map_or(ptr::null(), |x| x.node);
        let predicate = predicate.into().map_or(ptr::null(), |x| x.node);
        let object = object.into().map_or(ptr::null(), |x| x.node);
        let node = unsafe { lilv_world_get(*self.0.write().unwrap(), subject, predicate, object) };
        if node.is_null() {
            None
        } else {
            Some(ref_node(self, node))
        }
    }

    fn ask<'a, S, P, O>(&self, subject: S, predicate: P, object: O) -> bool
    where
        S: Into<Option<&'a Node<'a>>>,
        P: Into<Option<&'a Node<'a>>>,
        O: Into<Option<&'a Node<'a>>>,
    {
        let subject = subject.into().map_or(ptr::null(), |x| x.node);
        let predicate = predicate.into().map_or(ptr::null(), |x| x.node);
        let object = object.into().map_or(ptr::null(), |x| x.node);
        unsafe { lilv_world_ask(*self.0.write().unwrap(), subject, predicate, object) }
    }

    fn symbol<'a>(&self, subject: &Node<'a>) -> Option<Node> {
        let node = unsafe { lilv_world_get_symbol(*self.0.write().unwrap(), subject.node) };
        if node.is_null() {
            None
        } else {
            Some(new_node(self, node))
        }
    }

    /// Create a new URI value.
    fn new_uri<'a>(&self, uri: &CStr) -> Node<'a> {
        new_node(self, unsafe {
            lilv_new_uri(*self.0.write().unwrap(), uri.as_ptr())
        })
    }

    fn new_file_uri<'a>(&self, host: Option<&CStr>, path: &CStr) -> Node<'a> {
        new_node(self, unsafe {
            lilv_new_file_uri(
                *self.0.write().unwrap(),
                host.map_or(ptr::null(), |x| x.as_ptr()),
                path.as_ptr(),
            )
        })
    }

    fn new_string(&self, str: &CStr) -> Node {
        new_node(self, unsafe {
            lilv_new_string(*self.0.write().unwrap(), str.as_ptr())
        })
    }

    fn new_int(&self, value: i32) -> Node {
        new_node(self, unsafe {
            lilv_new_int(*self.0.write().unwrap(), value)
        })
    }

    fn new_float(&self, value: f32) -> Node {
        new_node(self, unsafe {
            lilv_new_float(*self.0.write().unwrap(), value)
        })
    }

    fn new_bool(&self, value: bool) -> Node {
        new_node(self, unsafe {
            lilv_new_bool(*self.0.write().unwrap(), value)
        })
    }
}

impl Drop for World {
    fn drop(&mut self) {
        unsafe { lilv_world_free(*self.0.write().unwrap()) }
    }
}
