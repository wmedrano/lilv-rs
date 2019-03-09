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
pub use crate::world::*;

pub use lv2_raw::LV2Descriptor;
pub use lv2_raw::LV2Feature;

type Void = libc::c_void;

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
