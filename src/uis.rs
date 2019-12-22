use crate::collection::*;
use crate::node::Node;
use crate::plugin::Plugin;
use crate::ui::UI;
use lilv_sys as lib;
use std::ptr::NonNull;

pub type UIs<'a> = Collection<lib::LilvUIs, lib::LilvUI, UI<'a>, &'a Plugin>;

impl<'a> UIs<'a> {
    pub fn size(&self) -> usize {
        unsafe { lib::lilv_uis_size(self.inner.read().as_ptr()) as _ }
    }

    pub fn get_by_uri(&self, uri: &Node) -> Option<UI> {
        let inner = self.inner.read().as_ptr();
        let uri = uri.inner.read().as_ptr();

        Some(UI::new_borrowed(
            NonNull::new(unsafe { lib::lilv_plugins_get_by_uri(inner, uri) as _ })?,
            self.owner,
        ))
    }
}

impl<'a> CollectionTrait for UIs<'a> {
    type Inner = lib::LilvUIs;
    type InnerTarget = lib::LilvUI;
    type Target = UI<'a>;
    type Owner = &'a Plugin;

    unsafe fn inner(&self) -> *const Self::Inner {
        self.inner.read().as_ptr()
    }

    fn begin_fn() -> BeginFn<Self> {
        lib::lilv_uis_begin
    }

    fn is_end_fn() -> IsEndFn<Self> {
        lib::lilv_uis_is_end
    }

    fn get_fn() -> GetFn<Self> {
        lib::lilv_uis_get
    }

    fn next_fn() -> NextFn<Self> {
        lib::lilv_uis_next
    }

    fn free_fn() -> FreeFn<Self> {
        lib::lilv_uis_free
    }

    fn get(&self, i: *mut lib::LilvIter) -> Self::Target {
        UI::new_borrowed(
            NonNull::new(unsafe { Self::get_fn()(self.inner(), i) as _ }).unwrap(),
            self.owner,
        )
    }
}
