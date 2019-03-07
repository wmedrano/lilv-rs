use crate::node::Any;
use crate::node::Bool;
use crate::node::Float;
use crate::node::Int;
use crate::node::Node;
use crate::node::Uri;
use crate::plugins::Plugins;
use crate::Void;
use std::ffi::CStr;
use std::marker::PhantomData;
use std::ptr;
use std::rc::Rc;
use std::sync::RwLock;

#[link(name = "lilv-0")]
extern "C" {
    fn lilv_world_new() -> *mut Void;
    fn lilv_world_set_option(world: *mut Void, uri: *const i8, value: *const Void);
    fn lilv_world_free(world: *mut Void);
    fn lilv_world_load_all(world: *mut Void);
    fn lilv_world_load_bundle(world: *mut Void, bundle_uri: *const Void);
    fn lilv_world_load_specifications(world: *mut Void);
    fn lilv_world_load_plugin_classes(world: *mut Void);
    fn lilv_world_unload_bundle(world: *mut Void, bundle_uri: *const Void) -> i32;
    fn lilv_world_load_resource(world: *mut Void, resource: *const Void) -> i32;
    fn lilv_world_unload_resource(world: *mut Void, resource: *const Void) -> i32;
    fn lilv_world_get_plugin_class(world: *mut Void) -> *const Void;
    fn lilv_world_get_plugin_classes(world: *mut Void) -> *const Void;
    fn lilv_world_get_all_plugins(world: *mut Void) -> *const Void;
    fn lilv_world_find_nodes(
        world: *mut Void,
        subject: *const Void,
        predicate: *const Void,
        object: *const Void,
    ) -> *mut Void;
    fn lilv_world_get(
        world: *mut Void,
        subject: *const Void,
        predicate: *const Void,
        object: *const Void,
    ) -> *mut Void;
    fn lilv_world_ask(
        world: *mut Void,
        subject: *const Void,
        predicate: *const Void,
        object: *const Void,
    ) -> u8;
    fn lilv_world_get_symbol(world: *mut Void, subject: *const Void) -> *mut Void;
    fn lilv_new_uri(world: *mut Void, uri: *const i8) -> *mut Void;
    fn lilv_new_file_uri(world: *mut Void, host: *const i8, path: *const i8) -> *mut Void;
    fn lilv_new_string(world: *mut Void, str: *const i8) -> *mut Void;
    fn lilv_new_int(world: *mut Void, val: i32) -> *mut Void;
    fn lilv_new_float(world: *mut Void, val: f32) -> *mut Void;
    fn lilv_new_bool(world: *mut Void, val: u8) -> *mut Void;
}

pub struct World(pub(crate) RwLock<*mut Void>);

pub(crate) fn new_node<T>(world: &Rc<World>, node: *mut Void) -> Node<T> {
    Node {
        node,
        world: world.clone(),
        owned: true,
        _phantom: PhantomData,
    }
}

pub(crate) fn ref_node<T>(world: &Rc<World>, node: *const Void) -> Node<T> {
    Node {
        node: node as *mut Void,
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
}

pub trait WorldImpl {
    fn set_option(&self, uri: &CStr, value: &Node<Any>);
    fn load_all(&self);
    fn load_bundle(&self, bundle_uri: &Node<Uri>);
    fn load_specifications(&self);
    fn load_plugin_classes(&self);
    fn unload_bundle(&self, bundle_uri: &Node<Uri>) -> Result<(), ()>;
    fn load_resource(&self, bundle_uri: &Node<Any>) -> Result<usize, ()>;
    fn unload_resource(&self, bundle_uri: &Node<Any>) -> Result<(), ()>;
    //fn get_plugin_class(&self) -> PluginClass;
    //fn get_plugin_classes(&self) -> PluginClasses;
    fn get_all_plugins(&self) -> Plugins;
    //fn find_nodes<S, P, O>(&self, subject: S, predicate: P, object: O) -> Nodes;
    //where
    //    S: Into<Option<Node<Any>>>,
    //    P: Into<Option<Node<Any>>>,
    //    O: Into<Option<Node<Any>>>;
    fn get<S, P, O>(&self, subject: S, predicate: P, object: O) -> Option<Node<Any>>
    where
        S: Into<Option<Node<Any>>>,
        P: Into<Option<Node<Any>>>,
        O: Into<Option<Node<Any>>>;
    fn ask<S, P, O>(&self, subject: S, predicate: P, object: O) -> bool
    where
        S: Into<Option<Node<Any>>>,
        P: Into<Option<Node<Any>>>,
        O: Into<Option<Node<Any>>>;
    fn get_symbol(&self, subject: &Node<Any>) -> Option<Node<crate::node::String>>;
    fn new_uri(&self, uri: &CStr) -> Node<Uri>;
    fn new_string(&self, str: &CStr) -> Node<crate::node::String>;
    fn new_int(&self, value: i32) -> Node<Int>;
    fn new_float(&self, value: f32) -> Node<Float>;
    fn new_bool(&self, value: bool) -> Node<Bool>;
}

impl WorldImpl for Rc<World> {
    /// Set an option for the world.
    ///
    /// Currently recognized options:
    /// * LILV_OPTION_FILTER_LANG
    /// * LILV_OPTION_DYN_MANIFEST
    fn set_option(&self, uri: &CStr, value: &Node<Any>) {
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

    fn load_bundle(&self, bundle_uri: &Node<Uri>) {
        unsafe { lilv_world_load_bundle(*self.0.write().unwrap(), bundle_uri.node) }
    }

    fn load_specifications(&self) {
        unsafe { lilv_world_load_specifications(*self.0.write().unwrap()) }
    }

    fn load_plugin_classes(&self) {
        unsafe { lilv_world_load_plugin_classes(*self.0.write().unwrap()) }
    }

    fn unload_bundle(&self, bundle_uri: &Node<Uri>) -> Result<(), ()> {
        if unsafe { lilv_world_unload_bundle(*self.0.write().unwrap(), bundle_uri.node) == 0 } {
            Ok(())
        } else {
            Err(())
        }
    }

    fn load_resource(&self, resource: &Node<Any>) -> Result<usize, ()> {
        unsafe {
            match lilv_world_load_resource(*self.0.write().unwrap(), resource.node) {
                -1 => Err(()),
                count => Ok(count as usize),
            }
        }
    }

    fn unload_resource(&self, resource: &Node<Any>) -> Result<(), ()> {
        if unsafe { lilv_world_unload_resource(*self.0.write().unwrap(), resource.node) == 0 } {
            Ok(())
        } else {
            Err(())
        }
    }

    /// Return a list of all found plugins.
    /// The returned list contains just enough references to query
    /// or instantiate plugins.  The data for a particular plugin will not be
    /// loaded into memory until a call to a method on [`Plugin`](struct.Plugin.html) results in
    /// a query (at which time the data is cached with the [`Plugin`](struct.Plugin.html) so future
    /// queries are very fast).
    fn get_all_plugins(&self) -> Plugins {
        Plugins {
            plugins: unsafe { lilv_world_get_all_plugins(*self.0.write().unwrap()) },
            world: self.clone(),
        }
    }

    fn get<S, P, O>(&self, subject: S, predicate: P, object: O) -> Option<Node<Any>>
    where
        S: Into<Option<Node<Any>>>,
        P: Into<Option<Node<Any>>>,
        O: Into<Option<Node<Any>>>,
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

    fn ask<S, P, O>(&self, subject: S, predicate: P, object: O) -> bool
    where
        S: Into<Option<Node<Any>>>,
        P: Into<Option<Node<Any>>>,
        O: Into<Option<Node<Any>>>,
    {
        let subject = subject.into().map_or(ptr::null(), |x| x.node);
        let predicate = predicate.into().map_or(ptr::null(), |x| x.node);
        let object = object.into().map_or(ptr::null(), |x| x.node);
        unsafe { lilv_world_ask(*self.0.write().unwrap(), subject, predicate, object) != 0 }
    }

    fn get_symbol(&self, subject: &Node<Any>) -> Option<Node<crate::node::String>> {
        let node = unsafe { lilv_world_get_symbol(*self.0.write().unwrap(), subject.node) };
        if node.is_null() {
            None
        } else {
            Some(new_node(self, node))
        }
    }

    /// Create a new URI value.
    fn new_uri(&self, uri: &CStr) -> Node<Uri> {
        new_node(self, unsafe {
            lilv_new_uri(*self.0.write().unwrap(), uri.as_ptr())
        })
    }

    fn new_string(&self, str: &CStr) -> Node<crate::node::String> {
        new_node(self, unsafe {
            lilv_new_string(*self.0.write().unwrap(), str.as_ptr())
        })
    }

    fn new_int(&self, value: i32) -> Node<Int> {
        new_node(self, unsafe {
            lilv_new_int(*self.0.write().unwrap(), value)
        })
    }

    fn new_float(&self, value: f32) -> Node<Float> {
        new_node(self, unsafe {
            lilv_new_float(*self.0.write().unwrap(), value)
        })
    }

    fn new_bool(&self, value: bool) -> Node<Bool> {
        new_node(self, unsafe {
            lilv_new_bool(*self.0.write().unwrap(), if value { 1 } else { 0 })
        })
    }
}

impl Drop for World {
    fn drop(&mut self) {
        unsafe { lilv_world_free(*self.0.write().unwrap()) }
    }
}
