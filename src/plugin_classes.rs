use crate::collection::*;
use crate::node::Node;
use crate::plugin_class::PluginClass;
use crate::world::InnerWorld;
use lilv_sys as lib;
use std::ptr::NonNull;
use std::sync::Arc;

pub type PluginClasses = Collection<lib::LilvPluginClasses, lib::LilvPluginClass, PluginClass>;

impl PluginClasses {
    pub fn size(&self) -> usize {
        unsafe { lib::lilv_plugin_classes_size(self.inner.read().as_ptr()) as _ }
    }

    pub fn get_by_uri(&self, uri: &Node) -> Option<PluginClass> {
        let inner = self.inner.read().as_ptr();
        let uri = uri.inner.read().as_ptr();

        Some(PluginClass::new_borrowed(
            NonNull::new(unsafe { lib::lilv_plugin_classes_get_by_uri(inner, uri) as _ })?,
            self.owner.clone(),
        ))
    }
}

impl CollectionTrait for PluginClasses {
    type Inner = lib::LilvPluginClasses;
    type InnerTarget = lib::LilvPluginClass;
    type Target = PluginClass;
    type Owner = Arc<InnerWorld>;

    unsafe fn inner(&self) -> *const Self::Inner {
        self.inner.read().as_ptr()
    }

    fn begin_fn() -> BeginFn<Self> {
        lib::lilv_plugin_classes_begin
    }

    fn is_end_fn() -> IsEndFn<Self> {
        lib::lilv_plugin_classes_is_end
    }

    fn get_fn() -> GetFn<Self> {
        lib::lilv_plugin_classes_get
    }

    fn next_fn() -> NextFn<Self> {
        lib::lilv_plugin_classes_next
    }

    fn free_fn() -> FreeFn<Self> {
        lib::lilv_plugin_classes_free
    }

    fn get(&self, i: *mut lib::LilvIter) -> Self::Target {
        PluginClass::new_borrowed(
            NonNull::new(unsafe { Self::get_fn()(self.inner(), i) as _ }).unwrap(),
            self.owner.clone(),
        )
    }
}
