use crate::collection::Collection;
use crate::collection::Iter;
use crate::node::Node;
use crate::ui::UI;
use crate::world::World;
use crate::Void;
use std::rc::Rc;

#[link(name = "lilv-0")]
extern "C" {
    fn lilv_uis_free(uis: *const Void);
    fn lilv_uis_size(uis: *const Void) -> u32;
    fn lilv_uis_begin(uis: *const Void) -> *mut Void;
    fn lilv_uis_get(uis: *const Void, i: *mut Void) -> *const Void;
    fn lilv_uis_next(uis: *const Void, i: *mut Void) -> *mut Void;
    fn lilv_uis_is_end(uis: *const Void, i: *mut Void) -> u8;
    fn lilv_uis_get_by_uri(uis: *const Void, uri: *const Void) -> *const Void;
}

pub struct UIs {
    pub(crate) uis: *const Void,
    pub(crate) owned: bool,
    pub(crate) world: Rc<World>,
}

impl Drop for UIs {
    fn drop(&mut self) {
        if self.owned {
            unsafe { lilv_uis_free(self.uis) }
        }
    }
}

impl AsRef<*const Void> for UIs {
    fn as_ref(&self) -> &*const Void {
        &self.uis
    }
}

impl<'a> Collection<'a> for UIs
where
    Self: 'a,
{
    type Target = UI;

    fn get(&self, i: *mut Void) -> Self::Target {
        UI {
            ui: unsafe { lilv_uis_get(self.uis, i) as *mut _ },
            world: self.world.clone(),
        }
    }
}

impl UIs {
    pub fn get_by_uri<'a>(&'a self, uri: &Node) -> Option<UI> {
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

    pub fn iter<'a>(&'a self) -> Iter<'a, Self> {
        Iter::new(self, lilv_uis_begin, lilv_uis_is_end, lilv_uis_next)
    }

    pub fn size(&self) -> usize {
        unsafe { lilv_uis_size(self.uis) as usize }
    }
}
