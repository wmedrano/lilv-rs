use std::{ptr::NonNull, sync::Arc};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};

use lilv_sys as lib;

use lv2_raw::LV2Feature;

use crate::instance::Instance;
use crate::node::Node;
use crate::plugin::Plugin;
use crate::world::Life;

unsafe impl Send for State {}
unsafe impl Sync for State {}

pub type Value = *const c_void;

/// Convert an objet to a generic port value.
pub fn value<T>(obj: &T) -> Value {
    NonNull::from(obj).as_ptr().cast()
}

/// Convert a generic port value to an objet. The requested data type must be the same as that of the corresponding port.
pub fn from_value<T>(value: &mut Value) -> &mut T {
    unsafe { &mut *(*value as *mut T) }
}

/// GetPortValue is the trait the user must implement to create a state from a plugin instance with [`crate::plugin::Plugin::new_state_from_instance()`].
pub trait GetPortValue {
    /// Function to get a port value.
    /// This function must return the pointer, the size and the URID of the type of the value of the port designated by `port_symbol`.
    fn get_port_value(&mut self, port_symbol: &str) -> (Value, u32, u32);
}

unsafe extern "C" fn get_port_value_func(
    port_symbol: *const c_char,
    user_data: *mut c_void,
    size: *mut u32,
    type_: *mut u32,
) -> *const c_void {
    let user_ptr = user_data as *mut Option<&mut dyn GetPortValue>;
    let user = unsafe { &mut *user_ptr };
    let port_symbol = unsafe { CStr::from_ptr(port_symbol) };

    if let Some(user) = user {
        let (val, sz, tp) = user.get_port_value(port_symbol.to_str().unwrap());

        *size = sz;
        *type_ = tp;

        return val;
    }
    *size = 0;
    *type_ = 0;
    std::ptr::null()
}

/// SetPortValue is the trait the user must implement to restore a plugin instance state with [`State::restore()`].
pub trait SetPortValue {
    /// Function to set a port value.
    /// This function must set the value of the port designated by `port_symbol` with the `value`, `size` and `type` parameters.
    fn set_port_value(&mut self, port_symbol: &str, value: Value, size: u32, type_: u32);
}

unsafe extern "C" fn set_port_value_func(
    port_symbol: *const c_char,
    user_data: *mut c_void,
    value: *const c_void,
    size: u32,
    type_: u32,
) {
    let user_ptr = user_data as *mut Option<&mut dyn SetPortValue>;
    let user = unsafe { &mut *user_ptr };
    let port_symbol = unsafe { CStr::from_ptr(port_symbol) };

    if let Some(user) = user {
        user.set_port_value(port_symbol.to_str().unwrap(), value, size, type_);
    }
}

#[derive(Clone)]
pub struct State {
    pub(crate) world: Arc<Life>,
    pub(crate) inner: NonNull<lib::LilvState>,
}

impl State {
    pub(crate) fn new(world: Arc<Life>, state_ptr: *mut lib::LilvState) -> Option<State> {
        Some(State {
            world,
            inner: NonNull::new(state_ptr)?,
        })
    }

    pub(crate) fn new_from_instance<'a, FS>(
        plugin: &Plugin,
        instance: &Instance,
        map: &mut lv2_raw::LV2UridMap,
        file_dir: Option<&str>,
        copy_dir: Option<&str>,
        link_dir: Option<&str>,
        save_dir: Option<&str>,
        user: Option<&mut dyn GetPortValue>,
        flags: lv2_sys::LV2_State_Flags,
        features: FS,
    ) -> Option<State>
    where
        FS: IntoIterator<Item = &'a LV2Feature>,
    {
        let plugin_ptr = plugin.inner.as_ptr();
        let instance = instance.inner.as_ptr();
        let file_d = CString::new(file_dir.unwrap_or_default()).unwrap();
        let file_dir: *const c_char = file_dir.map_or(std::ptr::null(), |_| file_d.as_ptr().cast());
        let copy_d = CString::new(copy_dir.unwrap_or_default()).unwrap();
        let copy_dir: *const c_char = copy_dir.map_or(std::ptr::null(), |_| copy_d.as_ptr().cast());
        let link_d = CString::new(link_dir.unwrap_or_default()).unwrap();
        let link_dir: *const c_char = link_dir.map_or(std::ptr::null(), |_| link_d.as_ptr().cast());
        let save_d = CString::new(save_dir.unwrap_or_default()).unwrap();
        let save_dir: *const c_char = save_dir.map_or(std::ptr::null(), |_| save_d.as_ptr().cast());
        let get_value: lib::LilvGetPortValueFunc = if user.is_some() {
            Some(get_port_value_func)
        }
        else {
            None
        };
        let user_data = NonNull::from(&user).as_ptr().cast();

        let features_vec: Vec<*const LV2Feature> = features
            .into_iter()
            .map(|f| f as *const LV2Feature)
            .chain(std::iter::once(std::ptr::null()))
            .collect();

        let state_ptr = unsafe {
            lib::lilv_state_new_from_instance(
                plugin_ptr,
                instance,
                map,
                file_dir,
                copy_dir,
                link_dir,
                save_dir,
                get_value,
                user_data,
                flags.0,
                features_vec.as_ptr(),
            )};

        State::new(plugin.life.clone(), state_ptr)
    }

    /// Return the number of properties in the state.
    pub fn num_properties(&self) -> u32 {
        unsafe { lib::lilv_state_get_num_properties(self.inner.as_ptr()) }
    }

    /// Get the URI of the plugin the state applies to.
    pub fn plugin_uri(&self) -> Node {
        let node_ptr = unsafe { lib::lilv_state_get_plugin_uri(self.inner.as_ptr()) }.cast_mut();
        let node = Node {
            inner: NonNull::new(node_ptr).unwrap(),
            borrowed: true,
            life: self.world.clone(),
        };
        node.clone()
    }

    /// Get the URI of the state.
    /// 
    /// This may return None if the state has not been saved and has no URI.
    pub fn uri(&self) -> Option<Node> {
        let node_ptr = unsafe { lib::lilv_state_get_uri(self.inner.as_ptr()) }.cast_mut();
        Some(Node {
            inner: NonNull::new(node_ptr)?,
            borrowed: true,
            life: self.world.clone(),
        })
    }

    /// Get the label of the state.
    pub fn label(&self) -> Option<&str> {
        let label_ptr = unsafe { lib::lilv_state_get_label(self.inner.as_ptr()) };

        if label_ptr.is_null() {
            return None;
        }
        Some(unsafe { CStr::from_ptr(label_ptr).to_str().ok()?})
    }

    /// Set the label of the state.
    pub fn set_label(&mut self, label: &str) {
        let label = CString::new(label).unwrap();
        unsafe { lib::lilv_state_set_label(self.inner.as_ptr(), label.as_ptr()); }
    }

    /// Set a metadata property on the state.
    /// 
    /// This is a generic version of [`set_label`](#method.set_label), which sets metadata
    /// properties visible to hosts, but not plugins.  This allows storing useful
    /// information such as comments or preset banks.
    pub fn set_metadata<T>(
        &mut self,
        key: u32,
        value: &T,
        size: usize,
        type_: u32,
        flags: lv2_sys::LV2_State_Flags,
    ) -> Result<(), lv2_sys::LV2_State_Status> {
        let value = NonNull::from(value).as_ptr().cast();
        let status = unsafe { lib::lilv_state_set_metadata(
            self.inner.as_ptr(),
            key,
            value,
            size,
            type_,
            flags.0,
        ) as lv2_sys::LV2_State_Status };

        if status == lv2_sys::LV2_State_Status_LV2_STATE_SUCCESS {
            return Ok(());
        }
        Err(status)
    }

    /// Enumerate the port values in a state snapshot.
    /// 
    /// This function is a subset of [`restore`](#method.restore) that only fires the
    /// `user`[`crate::state::SetPortValue::set_port_value()`] callback and does not directly affect a plugin instance.
    /// This is useful in hosts that need to retrieve the port values in a state snapshot for special handling.
    pub fn emit_port_values(
        &self,
        user: &mut dyn SetPortValue,
    ) {
        let set_value: lib::LilvSetPortValueFunc = Some(set_port_value_func);
        let some_user = Some(user);
        let user_data = NonNull::from(&some_user).as_ptr().cast();

        unsafe { lib::lilv_state_emit_port_values(self.inner.as_ptr(), set_value, user_data); }
    }

    /// Restore a plugin instance from a state snapshot.
    /// 
    /// This will set all the properties of `instance`, if given, to the values
    /// stored in the state.  If `user` is provided, [`crate::state::SetPortValue::set_port_value()`] will be called to restore each port value, otherwise the host must
    /// restore the port values itself (using [`emit_port_values`](#method.emit_port_values)) in order
    /// to completely restore the state.
    /// 
    /// If the state has properties and `instance` is given, this function is in
    /// the \"instantiation\" threading class, i.e. it MUST NOT be called
    /// simultaneously with any function on the same plugin instance.
    /// 
    /// If the state has no properties, only port values are set via [`crate::state::SetPortValue::set_port_value()`].
    /// 
    /// See <a href=https://lv2plug.in/ns/ext/state>LV2 state</a> from the
    /// LV2 State extension for details on the `flags` and `features` parameters.
    pub fn restore<'a, FS>(
        &self,
        instance: &Instance,
        user: Option<&mut dyn SetPortValue>,
        flags: lv2_sys::LV2_State_Flags,
        features: FS,
    )
    where
        FS: IntoIterator<Item = &'a LV2Feature>,
    {
        let instance = instance.inner.as_ptr();
        let set_value: lib::LilvSetPortValueFunc = if user.is_some() {
            Some(set_port_value_func)
        }
        else {
            None
        };
        let user_data = NonNull::from(&user).as_ptr().cast();

        let features_vec: Vec<*const LV2Feature> = features
            .into_iter()
            .map(|f| f as *const LV2Feature)
            .chain(std::iter::once(std::ptr::null()))
            .collect();

        unsafe { lib::lilv_state_restore(
            self.inner.as_ptr(),
            instance,
            set_value,
            user_data,
            flags.0,
            features_vec.as_ptr(),
        ); }
    }

    /// Save state to a file.
    /// 
    /// This function save the state to the `filename` Path relative to the `dir` Path of the bundle directory.
    /// 
    /// The format of state on disk is compatible with that defined in the LV2
    /// preset extension, i.e. this function may be used to save presets which can
    /// be loaded by any host.
    /// 
    /// If `uri` is None, the preset URI will be a file URI, but the bundle
    /// can safely be moved (i.e. the state file will use \"<>\" as the subject).
    pub fn save(
        &self,
        map: &mut lv2_raw::LV2UridMap,
        unmap: &mut lv2_sys::LV2_URID_Unmap,
        uri: Option<&str>,
        dir: &str,
        filename: &str,
    ) -> Result<(), lv2_sys::LV2_State_Status> {
        let world_ptr = self.world.inner.lock().as_ptr();
        let unmap = NonNull::from(unmap).as_ptr().cast();
        let c_uri = CString::new(uri.unwrap_or_default()).unwrap();
        let uri: *const c_char = uri.map_or(std::ptr::null(), |_| c_uri.as_ptr());
        let dir = CString::new(dir).unwrap();
        let filename = CString::new(filename).unwrap();

        let status = unsafe { lib::lilv_state_save(
            world_ptr,
            map,
            unmap,
            self.inner.as_ptr(),
            uri,
            dir.as_ptr(),
            filename.as_ptr(),
        ) as lv2_sys::LV2_State_Status };

        if status == lv2_sys::LV2_State_Status_LV2_STATE_SUCCESS {
            return Ok(());
        }
        Err(status)
    }

    /// Save state to a string.  This function does not use the filesystem.
    /// 
    /// The `base_uri` Base URI is for serialisation.  Unless you know what you are
    /// doing, pass None for this, otherwise the state may not be restorable via
    /// [`crate::world::World::new_state_from_string()`].
    pub fn to_string(
        &self,
        map: &mut lv2_raw::LV2UridMap,
        unmap: &mut lv2_sys::LV2_URID_Unmap,
        uri: &str,
        base_uri: Option<&str>,
    ) -> Option<String> {
        let world_ptr = self.world.inner.lock().as_ptr();
        let unmap = NonNull::from(unmap).as_ptr().cast();
        let uri = CString::new(uri).unwrap();
        let c_base_uri = CString::new(base_uri.unwrap_or_default()).unwrap();
        let base_uri: *const c_char = base_uri.map_or(std::ptr::null(), |_| c_base_uri.as_ptr().cast());

        let state_string = unsafe { lib::lilv_state_to_string(
            world_ptr,
            map,
            unmap,
            self.inner.as_ptr(),
            uri.as_ptr(),
            base_uri,
        ) };

        if state_string.is_null() {
            return None;
        }

        let result = unsafe{ CStr::from_ptr(state_string).to_string_lossy().to_string() };
        unsafe{ lib::lilv_free(state_string.cast()); }

        Some(result)
    }

    /// Unload a state from the world and delete all associated files.
    /// 
    /// This function DELETES FILES/DIRECTORIES FROM THE FILESYSTEM!  It is intended
    /// for removing user-saved presets, but can delete any state the user has
    /// permission to delete, including presets shipped with plugins.
    /// 
    /// The rdfs:seeAlso file for the state will be removed.  The entry in the
    /// bundle's manifest.ttl is removed, and if this results in an empty manifest,
    /// then the manifest file is removed.  If this results in an empty bundle, then
    /// the bundle directory is removed as well.
    pub fn delete(&self) -> Result<(), lv2_sys::LV2_State_Status> {
        let world_ptr = self.world.inner.lock().as_ptr();

        let status = unsafe { lib::lilv_state_delete(
            world_ptr,
            self.inner.as_ptr(),
        ) as lv2_sys::LV2_State_Status };

        if status == lv2_sys::LV2_State_Status_LV2_STATE_SUCCESS {
            return Ok(());
        }
        Err(status)
    }

    /// Get the underlying pointer to the the state.
    pub fn as_ptr(&self) -> *mut lib::LilvState {
        self.inner.as_ptr()
    }
}

impl Drop for State {
    fn drop(&mut self) {
        unsafe {
            lib::lilv_state_free(self.inner.as_ptr());
        }
    }
}

impl PartialEq for State {
    /// Return true iff `self` is equivalent to `other`.
    fn eq(&self, other: &Self) -> bool {
        unsafe { lib::lilv_state_equals(self.inner.as_ptr(), other.inner.as_ptr()) }
    }
}


#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::convert::TryFrom;
    use std::ffi::{CStr, CString};
    use std::ptr::NonNull;

    use lv2_raw::LV2Feature;

    use crate::world::World;

    type MapImpl = HashMap<CString, u32>;

    extern "C" fn do_map(handle: lv2_raw::LV2UridMapHandle, uri_ptr: *const i8) -> lv2_raw::LV2Urid {
        let handle = handle as *mut MapImpl;
        let map = unsafe { &mut *handle };
        let uri = unsafe { CStr::from_ptr(uri_ptr) };
    
        if let Some(id) = map.get(uri) {
            return *id;
        }
        let id = u32::try_from(map.len()).expect("URID space has exceeded capacity for u32.") + 1;
        map.insert(uri.to_owned(), id);
        id
    }

    extern "C" fn do_unmap(handle: lv2_sys::LV2_URID_Unmap_Handle, urid: lv2_sys::LV2_URID) -> *const i8 {
        let handle: *const MapImpl = handle as *const _;
        let map = unsafe { &*handle };

        for (uri, id) in map.iter() {
            if *id == urid {
                return uri.as_ptr();
            }
        }
        std::ptr::null()
    }

    #[test]
    fn test_new_from_world() {
        let world = World::with_load_all();
        let map = MapImpl::new();
        let map_ptr = NonNull::from(&map);

        let mut lv2_urid_map = lv2_raw::LV2UridMap {
            handle: map_ptr.as_ptr().cast(),
            map: do_map,
        };

        let subject = world.new_uri("http://lv2plug.in/plugins/eg-amp");

        let state = world.new_state(&mut lv2_urid_map, &subject);
        assert!(state.is_some());
    }

    #[test]
    fn test_new_from_file() {
        let world = World::with_load_all();
        let map = MapImpl::new();
        let map_ptr = NonNull::from(&map);

        let mut lv2_urid_map = lv2_raw::LV2UridMap {
            handle: map_ptr.as_ptr().cast(),
            map: do_map,
        };
        let mut lv2_urid_unmap = lv2_sys::LV2_URID_Unmap {
            handle: map_ptr.as_ptr().cast(),
            unmap: Some(do_unmap),
        };
        let map_data_ptr = NonNull::from(&lv2_urid_map);
        let urid_map_feature = LV2Feature {
            uri: lv2_sys::LV2_URID__map.as_ptr().cast(),
            data: map_data_ptr.as_ptr().cast(),
        };

        let unmap_data_ptr = NonNull::from(&lv2_urid_unmap);
        let urid_unmap_feature = LV2Feature {
            uri: lv2_sys::LV2_URID__unmap.as_ptr().cast(),
            data: unmap_data_ptr.as_ptr().cast(),
        };

        let features = vec![urid_map_feature, urid_unmap_feature];
        let plugin_uri = "http://lv2plug.in/plugins/eg-amp";
        let plugin_uri_node = world.new_uri(plugin_uri);
        let plugin = world.plugins().plugin(&plugin_uri_node).unwrap();
        let instance = unsafe{ plugin.instantiate(44100., &features)};
        assert!(instance.is_some());
        let instance = instance.unwrap();

        let state = plugin.new_state_from_instance(
            &instance,
            &mut lv2_urid_map,
            None,
            None,
            None,
            None,
            None,
            lv2_sys::LV2_State_Flags::LV2_STATE_IS_PORTABLE,
            &features
        );

        let res = state.unwrap().save(&mut lv2_urid_map, &mut lv2_urid_unmap, Some(plugin_uri), ".", "filename");
        assert!(res == Ok(()));

        let subject = world.new_uri("http://lv2plug.in/plugins/eg-amp");
        let state = world.new_state_from_file(&mut lv2_urid_map, Some(&subject), "filename");
        assert!(state.is_some());
    }

    #[test]
    fn test_new_from_instance() {
        let world = World::with_load_all();
        let map = MapImpl::new();
        let map_ptr = NonNull::from(&map);

        let mut lv2_urid_map = lv2_raw::LV2UridMap {
            handle: map_ptr.as_ptr().cast(),
            map: do_map,
        };
        let mut lv2_urid_unmap = lv2_sys::LV2_URID_Unmap {
            handle: map_ptr.as_ptr().cast(),
            unmap: Some(do_unmap),
        };
        let map_data_ptr = NonNull::from(&lv2_urid_map);
        let urid_map_feature = LV2Feature {
            uri: lv2_sys::LV2_URID__map.as_ptr().cast(),
            data: map_data_ptr.as_ptr().cast(),
        };

        let unmap_data_ptr = NonNull::from(&lv2_urid_unmap);
        let urid_unmap_feature = LV2Feature {
            uri: lv2_sys::LV2_URID__unmap.as_ptr().cast(),
            data: unmap_data_ptr.as_ptr().cast(),
        };

        let features = vec![urid_map_feature, urid_unmap_feature];
        let plugin_uri = "http://lv2plug.in/plugins/eg-amp";
        let plugin_uri_node = world.new_uri(plugin_uri);
        let plugin = world.plugins().plugin(&plugin_uri_node).unwrap();
        let instance = unsafe{ plugin.instantiate(44100., &features)};
        assert!(instance.is_some());
        let instance = instance.unwrap();

        let state = plugin.new_state_from_instance(
            &instance,
            &mut lv2_urid_map,
            None,
            None,
            None,
            None,
            None,
            lv2_sys::LV2_State_Flags::LV2_STATE_IS_PORTABLE,
            &features
        );
        assert!(state.is_some());
        let mut state = state.unwrap();
        assert!(state.plugin_uri() == plugin_uri_node);
        assert!(state.uri().is_none());
        state.set_label("my_label");
        assert!(state.label() == Some("my_label"));

        let backup = state.to_string(&mut lv2_urid_map, &mut lv2_urid_unmap, plugin_uri, None);
        assert!(backup.is_some());
        let backup = backup.unwrap();

        let saved_state = world.new_state_from_string(&mut lv2_urid_map, &backup);
        assert!(saved_state.is_some());
        let saved_state = saved_state.unwrap();
        assert!(saved_state.uri() == Some(plugin_uri_node));
        saved_state.restore(&instance, None, lv2_sys::LV2_State_Flags::LV2_STATE_IS_PORTABLE, &features);
    }
}
