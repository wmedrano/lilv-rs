use crate::collection::*;
use crate::node::Node;
use crate::plugin::Plugin;
use crate::world::InnerWorld;
use lilv_sys as lib;
use std::ptr::NonNull;
use std::sync::Arc;

pub type Plugins = Collection<lib::LilvPlugins, lib::LilvPlugin, Plugin>;

impl Plugins {
    pub fn size(&self) -> usize {
        unsafe { lib::lilv_plugins_size(self.inner.read().as_ptr()) as _ }
    }

    pub fn get_by_uri(&self, uri: &Node) -> Option<Plugin> {
        let inner = self.inner.read().as_ptr();
        let uri = uri.inner.read().as_ptr();

        Some(Plugin::new_borrowed(
            NonNull::new(unsafe { lib::lilv_plugins_get_by_uri(inner, uri) as _ })?,
            self.owner.clone(),
        ))
    }
}

impl CollectionTrait for Plugins {
    type Inner = lib::LilvPlugins;
    type InnerTarget = lib::LilvPlugin;
    type Target = Plugin;
    type Owner = Arc<InnerWorld>;

    unsafe fn inner(&self) -> *const Self::Inner {
        self.inner.read().as_ptr()
    }

    fn begin_fn() -> BeginFn<Self> {
        lib::lilv_plugins_begin
    }

    fn is_end_fn() -> IsEndFn<Self> {
        lib::lilv_plugins_is_end
    }

    fn get_fn() -> GetFn<Self> {
        lib::lilv_plugins_get
    }

    fn next_fn() -> NextFn<Self> {
        lib::lilv_plugins_next
    }

    fn free_fn() -> FreeFn<Self> {
        fake_free::<Self>
    }

    fn get(&self, i: *mut lib::LilvIter) -> Self::Target {
        Plugin::new_borrowed(
            NonNull::new(unsafe { Self::get_fn()(self.inner(), i) as _ }).unwrap(),
            self.owner.clone(),
        )
    }
}
