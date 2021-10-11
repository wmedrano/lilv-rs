use crate::plugin::Plugin;
use crate::world::Life;
use lilv_sys as lib;
use parking_lot::RwLock;
use std::ptr::NonNull;
use std::sync::Arc;

/// An iterator over plugins.
pub struct PluginsIter {
    pub(crate) world: Arc<Life>,
    pub(crate) ptr: *const lib::LilvPlugins,
    pub(crate) iter: *mut lib::LilvIter,
}

impl Iterator for PluginsIter {
    type Item = Plugin;

    fn next(&mut self) -> Option<Plugin> {
        let ptr: *mut lib::LilvPlugin =
            unsafe { lib::lilv_plugins_get(self.ptr, self.iter) } as *mut _;
        self.iter = unsafe { lib::lilv_plugins_next(self.ptr, self.iter) };
        match NonNull::new(ptr) {
            Some(ptr) => Some(Plugin {
                world: self.world.clone(),
                inner: RwLock::new(ptr),
            }),
            None => None,
        }
    }
}
