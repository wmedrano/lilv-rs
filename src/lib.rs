//! Lilv is a simple yet powerful library for using LV2 plugins.
//!
//! For more information about LV2, see [`http://lv2plug.in`].
//!
//! For more information about Lilv, see [`http://drobilla.net/software/lilv`].
//!
//! [`http://lv2plug.in`]: http://lv2plug.in
//! [`http://drobilla.net/software/lilv`]: http://drobilla.net/software/lilv

mod collection;
mod instance;
mod node;
mod nodes;
mod plugin;
mod plugin_class;
mod plugin_classes;
mod plugins;
mod port;
mod scale_point;
mod scale_points;
mod state;
mod ui;
mod uis;
mod world;

pub use crate::collection::*;
pub use crate::instance::*;
pub use crate::node::*;
pub use crate::nodes::*;
pub use crate::plugin::*;
pub use crate::plugin_class::*;
pub use crate::plugin_classes::*;
pub use crate::plugins::*;
pub use crate::port::*;
pub use crate::scale_point::*;
pub use crate::scale_points::*;
pub use crate::state::*;
pub use crate::ui::*;
pub use crate::uis::*;
pub use crate::world::*;

pub use lv2_raw::LV2Descriptor;
pub use lv2_raw::LV2Feature;

use std::ffi::CStr;
use std::ffi::CString;
use std::ptr;

type Void = libc::c_void;

#[link(name = "lilv-0")]
extern "C" {
    fn lilv_free(value: *mut Void);
    fn lilv_file_uri_parse(uri: *const i8, hostname: *mut *mut i8) -> *mut i8;
}

#[link(name = "serd-0")]
extern "C" {
    fn serd_free(value: *mut Void);
}

pub fn file_uri_parse(uri: &CStr, with_hostname: bool) -> Option<(CString, Option<CString>)> {
    let mut hostname = ptr::null_mut();

    let path = unsafe {
        lilv_file_uri_parse(
            uri.as_ptr(),
            if with_hostname {
                &mut hostname
            } else {
                ptr::null_mut()
            },
        )
    };

    if path.is_null() {
        None
    } else {
        unsafe {
            let ret_path = CString::from(CStr::from_ptr(path));
            lilv_free(path as *mut Void);

            let ret_hostname = if with_hostname & !hostname.is_null() {
                let ret_hostname = CString::from(CStr::from_ptr(hostname));
                serd_free(hostname as *mut Void);
                Some(ret_hostname)
            } else {
                None
            };

            Some((ret_path, ret_hostname))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use std::ffi::CString;

    #[test]
    fn hello_world() {
        let w = World::new().unwrap();
        w.load_all();
        let plugins = w.get_all_plugins();
        let node = w.new_uri(&CString::new("http://lv2plug.in/plugins/eg-amp").unwrap());
        let _plugin = plugins.get_by_uri(&node);
    }
}
