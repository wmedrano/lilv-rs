use crate::instance::Instance;
use crate::instance::InstanceImpl;
use crate::node::Node;
use crate::plugin::Plugin;
use crate::world::ref_node;
use crate::world::World;
use crate::Void;
use lv2_raw::LV2Feature;
use std::ffi::CStr;
use std::ffi::CString;
use std::rc::Rc;

type LilvGetPortValueFunc = extern "C" fn(
    port_symbol: *const i8,
    user_data: *mut Void,
    size: *mut u32,
    tyep: *mut u32,
) -> *const Void;

type LilvSetPortValueFunc = extern "C" fn(
    port_symbol: *const i8,
    user_data: *mut Void,
    value: *const Void,
    size: u32,
    tyep: u32,
);

#[link(name = "lilv-0")]
extern "C" {
    fn lilv_state_new_from_world(world: *mut Void, map: *mut Void, node: *const Void) -> *mut Void;
    fn lilv_state_new_from_file(
        world: *mut Void,
        map: *mut Void,
        subject: *const Void,
        path: *const i8,
    ) -> *mut Void;
    fn lilv_state_new_from_string(world: *mut Void, map: *mut Void, str: *const i8) -> *mut Void;
    fn lilv_state_new_from_instance(
        plugin: *const Void,
        instance: *mut InstanceImpl,
        map: *mut Void,
        scratch_dir: *const i8,
        copy_dir: *const i8,
        link_dir: *const i8,
        save_dir: *const i8,
        get_value: LilvGetPortValueFunc,
        user_data: *mut Void,
        flags: u32,
        features: *const *const LV2Feature,
    ) -> *mut Void;
    fn lilv_state_free(state: *mut Void);
    fn lilv_state_equals(a: *const Void, b: *const Void) -> u8;
    fn lilv_state_get_num_properties(state: *const Void) -> u32;
    fn lilv_state_get_plugin_uri(state: *const Void) -> *const Void;
    fn lilv_state_get_uri(state: *const Void) -> *const Void;
    fn lilv_state_get_label(state: *const Void) -> *const i8;
    fn lilv_state_set_label(state: *mut Void, label: *const i8);
    fn lilv_state_set_metadata(
        state: *mut Void,
        key: u32,
        value: *const Void,
        size: usize,
        tyep: u32,
        flags: u32,
    ) -> i32;
    fn lilv_state_emit_port_values(
        state: *const Void,
        set_value: LilvSetPortValueFunc,
        user_data: *mut Void,
    );
    fn lilv_state_restore(
        state: *const Void,
        instance: *mut InstanceImpl,
        set_value: LilvSetPortValueFunc,
        user_data: *mut Void,
        flags: u32,
        features: *const *const LV2Feature,
    );
    fn lilv_state_save(
        world: *mut Void,
        map: *mut Void,
        unmap: *mut Void,
        state: *const Void,
        uri: *const i8,
        dir: *const i8,
        filename: *const i8,
    ) -> i32;
    fn lilv_state_to_string(
        world: *mut Void,
        map: *mut Void,
        unmap: *mut Void,
        state: *const Void,
        uri: *const i8,
        base_uri: *const i8,
    ) -> *mut i8;
    fn lilv_state_delete(world: *mut Void, state: *const Void) -> i32;
}

struct WrapFn<F>(F);

#[repr(C)]
struct FakeLV2UridMap {
    handle: *mut Void,
    map: extern "C" fn(*mut Void, *const i8) -> u32,
}

extern "C" fn fake_map<F>(handle: *mut Void, uri: *const i8) -> u32
where
    F: FnMut(&CStr) -> u32,
{
    unsafe {
        let f: &mut WrapFn<F> = std::mem::transmute(handle);
        f.0(CStr::from_ptr(uri))
    }
}

#[repr(C)]
struct FakeLV2UridUnmap {
    handle: *mut Void,
    unmap: extern "C" fn(*mut Void, u32) -> Option<*const i8>,
}

extern "C" fn fake_unmap<'a, F>(handle: *mut Void, urid: u32) -> Option<*const i8>
where
    F: FnMut(u32) -> Option<&'a CStr>,
{
    unsafe {
        let f: &mut WrapFn<F> = std::mem::transmute(handle);
        Some(f.0(urid)?.as_ptr())
    }
}

extern "C" fn fake_get_port_value<F>(
    port_symbol: *const i8,
    user_data: *mut Void,
    size: *mut u32,
    tyep: *mut u32,
) -> *const Void
where
    F: FnMut(&CStr) -> PortValue,
{
    unsafe {
        let f: &mut WrapFn<F> = std::mem::transmute(user_data);
        let ret = f.0(CStr::from_ptr(port_symbol));

        *size = ret.size;
        *tyep = ret.tyep;
        ret.value
    }
}

extern "C" fn fake_set_port_value<F>(
    port_symbol: *const i8,
    user_data: *mut Void,
    value: *const Void,
    size: u32,
    tyep: u32,
) where
    F: FnMut(&CStr, &PortValue),
{
    unsafe {
        let f: &mut WrapFn<F> = std::mem::transmute(user_data);
        f.0(
            CStr::from_ptr(port_symbol),
            &PortValue { value, size, tyep },
        )
    }
}

pub struct PortValue {
    pub value: *const libc::c_void,
    pub size: u32,
    pub tyep: u32,
}

pub struct State {
    pub(crate) state: *mut Void,
    pub(crate) world: Rc<World>,
    pub(crate) owned: bool,
}

impl State {
    pub fn new_from_world<F>(world: &Rc<World>, map: F, node: &Node) -> Self
    where
        F: FnMut(&CStr) -> u32,
    {
        let mut wrap = WrapFn(map);
        let mut fake = FakeLV2UridMap {
            handle: &mut wrap as *mut _ as *mut Void,
            map: fake_map::<F>,
        };

        State {
            state: unsafe {
                lilv_state_new_from_world(
                    *world.0.write().unwrap(),
                    &mut fake as *mut _ as *mut Void,
                    node.node,
                )
            },
            world: world.clone(),
            owned: true,
        }
    }

    pub fn new_from_file<Map>(world: &Rc<World>, map: Map, subject: &Node, path: &CStr) -> Self
    where
        Map: FnMut(&CStr) -> u32,
    {
        let mut wrap = WrapFn(map);
        let mut fake = FakeLV2UridMap {
            handle: &mut wrap as *mut _ as *mut Void,
            map: fake_map::<Map>,
        };

        State {
            state: unsafe {
                lilv_state_new_from_file(
                    *world.0.write().unwrap(),
                    &mut fake as *mut _ as *mut Void,
                    subject.node,
                    path.as_ptr(),
                )
            },
            world: world.clone(),
            owned: true,
        }
    }

    pub fn new_from_string<Map>(world: &Rc<World>, map: Map, str: &CStr) -> Self
    where
        Map: FnMut(&CStr) -> u32,
    {
        let mut wrap = WrapFn(map);
        let mut fake = FakeLV2UridMap {
            handle: &mut wrap as *mut _ as *mut Void,
            map: fake_map::<Map>,
        };

        State {
            state: unsafe {
                lilv_state_new_from_string(
                    *world.0.write().unwrap(),
                    &mut fake as *mut _ as *mut Void,
                    str.as_ptr(),
                )
            },
            world: world.clone(),
            owned: true,
        }
    }

    pub fn new_from_instance<'a, Map, GetValue, P3, P4, P5, P6>(
        plugin: &Plugin,
        instance: &mut Instance,
        map: Map,
        scratch_dir: P3,
        copy_dir: P4,
        link_dir: P5,
        save_dir: P6,
        get_value: GetValue,
        flags: u32,
        features: *const *const LV2Feature,
    ) -> State
    where
        Map: FnMut(&CStr) -> u32,
        GetValue: FnMut(&CStr) -> PortValue,
        P3: Into<Option<&'a CStr>>,
        P4: Into<Option<&'a CStr>>,
        P5: Into<Option<&'a CStr>>,
        P6: Into<Option<&'a CStr>>,
    {
        let (scratch_dir, copy_dir, link_dir, save_dir) = (
            scratch_dir.into(),
            copy_dir.into(),
            link_dir.into(),
            save_dir.into(),
        );

        let mut wrap_map = WrapFn(map);
        let mut wrap_get_value = WrapFn(get_value);

        let mut fake_map = FakeLV2UridMap {
            handle: &mut wrap_map as *mut _ as *mut Void,
            map: fake_map::<Map>,
        };

        State {
            state: unsafe {
                lilv_state_new_from_instance(
                    plugin.plugin,
                    instance.0,
                    &mut fake_map as *mut _ as *mut Void,
                    scratch_dir.map_or_else(std::ptr::null, |x| x.as_ptr()),
                    copy_dir.map_or_else(std::ptr::null, |x| x.as_ptr()),
                    link_dir.map_or_else(std::ptr::null, |x| x.as_ptr()),
                    save_dir.map_or_else(std::ptr::null, |x| x.as_ptr()),
                    fake_get_port_value::<GetValue>,
                    &mut wrap_get_value as *mut _ as *mut Void,
                    flags,
                    features,
                )
            },
            world: plugin.world.clone(),
            owned: true,
        }
    }

    pub fn get_num_properties(&self) -> u32 {
        unsafe { lilv_state_get_num_properties(self.state) }
    }

    pub fn get_plugin_uri(&self) -> Node {
        ref_node(&self.world, unsafe {
            lilv_state_get_plugin_uri(self.state)
        })
    }

    pub fn get_uri(&self) -> Option<Node> {
        let node = unsafe { lilv_state_get_uri(self.state) };
        if node.is_null() {
            None
        } else {
            Some(ref_node(&self.world, node))
        }
    }

    pub fn get_label(&self) -> &CStr {
        unsafe { CStr::from_ptr(lilv_state_get_label(self.state)) }
    }

    pub fn set_label(&self, label: &CStr) {
        unsafe { lilv_state_set_label(self.state, label.as_ptr()) }
    }

    pub fn set_metadata(
        &self,
        key: u32,
        value: *const libc::c_void,
        size: usize,
        tyep: u32,
        flags: u32,
    ) -> bool {
        unsafe { lilv_state_set_metadata(self.state, key, value, size, tyep, flags) == 0 }
    }

    pub fn emit_port_values<F>(&self, set_value: F)
    where
        F: FnMut(&CStr, &PortValue),
    {
        let mut wrap = WrapFn(set_value);
        unsafe {
            lilv_state_emit_port_values(
                self.state,
                fake_set_port_value::<F>,
                &mut wrap as *mut _ as *mut Void,
            )
        }
    }

    pub fn restore<F>(
        &self,
        instance: &mut Instance,
        set_value: F,
        flags: u32,
        features: *const *const LV2Feature,
    ) where
        F: FnMut(&CStr, &PortValue),
    {
        let mut wrap = WrapFn(set_value);
        unsafe {
            lilv_state_restore(
                self.state,
                instance.0,
                fake_set_port_value::<F>,
                &mut wrap as *mut _ as *mut Void,
                flags,
                features,
            )
        }
    }

    pub fn save<'a, Map, Unmap>(
        &self,
        map: Map,
        unmap: Unmap,
        uri: Option<&CStr>,
        path: &CStr,
        filename: &CStr,
    ) -> bool
    where
        Map: FnMut(&CStr) -> u32,
        Unmap: FnMut(u32) -> Option<&'a CStr>,
    {
        let mut wrap_map = WrapFn(map);
        let mut wrap_unmap = WrapFn(unmap);

        let mut fake_map = FakeLV2UridMap {
            handle: &mut wrap_map as *mut _ as *mut Void,
            map: fake_map::<Map>,
        };

        let mut fake_unmap = FakeLV2UridUnmap {
            handle: &mut wrap_unmap as *mut _ as *mut Void,
            unmap: fake_unmap::<Unmap>,
        };

        unsafe {
            lilv_state_save(
                *self.world.0.write().unwrap(),
                &mut fake_map as *mut _ as *mut Void,
                &mut fake_unmap as *mut _ as *mut Void,
                self.state,
                uri.map_or(std::ptr::null(), |x| x.as_ptr()),
                path.as_ptr(),
                filename.as_ptr(),
            ) == 0
        }
    }

    pub fn lilv_state_to_string<'a, Map, Unmap>(
        &self,
        map: Map,
        unmap: Unmap,
        uri: &CStr,
        base_uri: Option<&CStr>,
    ) -> CString
    where
        Map: FnMut(&CStr) -> u32,
        Unmap: FnMut(u32) -> Option<&'a CStr>,
    {
        let mut wrap_map = WrapFn(map);
        let mut wrap_unmap = WrapFn(unmap);

        let mut fake_map = FakeLV2UridMap {
            handle: &mut wrap_map as *mut _ as *mut Void,
            map: fake_map::<Map>,
        };

        let mut fake_unmap = FakeLV2UridUnmap {
            handle: &mut wrap_unmap as *mut _ as *mut Void,
            unmap: fake_unmap::<Unmap>,
        };

        unsafe {
            CString::from_raw(lilv_state_to_string(
                *self.world.0.write().unwrap(),
                &mut fake_map as *mut _ as *mut Void,
                &mut fake_unmap as *mut _ as *mut Void,
                self.state,
                uri.as_ptr(),
                base_uri.map_or(std::ptr::null(), |x| x.as_ptr()),
            ))
        }
    }

    pub fn delete(&self) -> bool {
        unsafe { lilv_state_delete(*self.world.0.write().unwrap(), self.state) == 0 }
    }
}

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        unsafe { lilv_state_equals(self.state, other.state) != 0 }
    }
}

impl Drop for State {
    fn drop(&mut self) {
        if self.owned {
            unsafe { lilv_state_free(self.state) }
        }
    }
}
