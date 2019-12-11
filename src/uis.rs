use crate::node::Node;
use crate::ui::UI;
use crate::world::World;
use lilv_sys::*;
use std::rc::Rc;

pub struct UIs {
    pub(crate) uis: *const LilvUIs,
    pub(crate) owned: bool,
    pub(crate) world: Rc<World>,
}

impl Drop for UIs {
    fn drop(&mut self) {
        if self.owned {
            unsafe { lilv_uis_free(self.uis as *mut LilvUIs) }
        }
    }
}

impl UIs {
    pub fn by_uri<'a>(&'a self, uri: &Node) -> Option<UI> {
        let ptr = unsafe { lilv_uis_get_by_uri(self.uis, uri.node) };

        if ptr.is_null() {
            None
        } else {
            Some(UI {
                ui: ptr as *mut _,
                world: self.world.clone(),
            })
        }
    }

    pub fn iter(&self) -> UiIter<'_> {
        UiIter {
            uis: self,
            iter: unsafe { lilv_uis_begin(self.uis) },
        }
    }

    pub fn size(&self) -> usize {
        unsafe { lilv_uis_size(self.uis) as usize }
    }
}

pub struct UiIter<'a> {
    uis: &'a UIs,
    iter: *mut LilvIter,
}

impl<'a> Iterator for UiIter<'a> {
    type Item = UI;

    fn next(&mut self) -> Option<UI> {
        let ptr = unsafe { lilv_uis_get(self.uis.uis, self.iter) };
        if ptr.is_null() {
            None
        } else {
            self.iter = unsafe { lilv_uis_next(self.uis.uis, self.iter) };
            Some(UI {
                ui: ptr as *mut LilvUI,
                world: self.uis.world.clone(),
            })
        }
    }
}
