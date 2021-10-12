use crate::node::Node;
use crate::plugin::Plugin;
use crate::ui::UI;
use crate::Life;
use lilv_sys as lib;
use std::ptr::NonNull;
use std::sync::Arc;

pub struct Uis<'a> {
    pub(crate) inner: NonNull<lib::LilvUIs>,
    pub(crate) plugin: &'a Plugin,
    pub(crate) life: Arc<Life>,
}

impl<'a> Uis<'a> {
    #[must_use]
    pub fn size(&self) -> usize {
        let _life = self.life.inner.read();
        unsafe { lib::lilv_uis_size(self.inner.as_ptr()) as _ }
    }

    #[must_use]
    pub fn get_by_uri(&self, uri: &Node) -> Option<UI> {
        let _life = self.life.inner.read();
        let inner = self.inner.as_ptr();
        let uri = uri.inner.as_ptr();

        Some(UI {
            inner: NonNull::new(unsafe { lib::lilv_plugins_get_by_uri(inner, uri) as _ })?,
            plugin: self.plugin,
            life: self.life.clone(),
        })
    }

    #[must_use]
    pub fn iter(&self) -> UiIter<'_> {
        let _life = self.life.inner.read();
        UiIter {
            uis: self,
            iter: unsafe { lib::lilv_uis_begin(self.inner.as_ptr()).cast() },
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
        let _life = self.uis.life.inner.read();
        let next =
            unsafe { lib::lilv_uis_get(self.uis.inner.as_ptr(), self.iter) } as *mut lib::LilvUI;
        let ret = Some(UI {
            inner: NonNull::new(next)?,
            plugin: self.uis.plugin,
            life: self.uis.life.clone(),
        });
        self.iter = unsafe { lib::lilv_uis_next(self.uis.inner.as_ptr(), self.iter) };
        ret
    }
}
