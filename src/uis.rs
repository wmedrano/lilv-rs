use crate::node::Node;
use crate::plugin::Plugin;
use crate::ui::UI;
use crate::InnerWorld;
use lilv_sys as lib;
use std::ptr::NonNull;
use std::sync::Arc;

pub struct Uis<'a> {
    pub(crate) inner: NonNull<lib::LilvUIs>,
    pub(crate) plugin: &'a Plugin,
    pub(crate) _world: Arc<InnerWorld>,
}

impl<'a> Uis<'a> {
    pub fn size(&self) -> usize {
        unsafe { lib::lilv_uis_size(self.inner.as_ptr()) as _ }
    }

    pub fn get_by_uri(&self, uri: &Node) -> Option<UI> {
        let inner = self.inner.as_ptr();
        let uri = uri.inner.read().as_ptr();

        Some(UI {
            inner: NonNull::new(unsafe { lib::lilv_plugins_get_by_uri(inner, uri) as _ })?,
            plugin: self.plugin,
        })
    }

    pub fn iter(&self) -> UiIter<'_> {
        UiIter {
            uis: self,
            iter: unsafe { lib::lilv_uis_begin(self.inner.as_ptr()) as _ },
        }
    }
}

pub struct UiIter<'a> {
    uis: &'a Uis<'a>,
    iter: *mut lib::LilvIter,
}

impl<'a> Iterator for UiIter<'a> {
    type Item = UI<'a>;

    fn next(&mut self) -> Option<UI<'a>> {
        let next =
            unsafe { lib::lilv_uis_get(self.uis.inner.as_ptr(), self.iter) } as *mut lib::LilvUI;
        let ret = Some(UI {
            inner: NonNull::new(next)?,
            plugin: self.uis.plugin,
        });
        self.iter = unsafe { lib::lilv_uis_next(self.uis.inner.as_ptr(), self.iter) };
        ret
    }
}
