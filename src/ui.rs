use crate::node::Node;
use crate::nodes::Nodes;
use crate::world::ref_node;
use crate::world::World;
use crate::Void;
use lilv_sys::*;
use std::ffi::CStr;
use std::ops::Deref;
use std::ptr;
use std::rc::Rc;

pub struct UI {
    pub(crate) ui: *mut LilvUI,
    pub(crate) world: Rc<World>,
}

impl UI {
    pub fn get_uri(&self) -> Node {
        ref_node(&self.world, unsafe { lilv_ui_get_uri(self.ui) })
    }

    pub fn get_classes(&self) -> Nodes {
        Nodes {
            nodes: unsafe { lilv_ui_get_classes(self.ui) as *mut Void },
            world: self.world.clone(),
            owned: false,
        }
    }

    pub fn is_a(&self, class_uri: &Node) -> bool {
        unsafe { lilv_ui_is_a(self.ui, class_uri.node) }
    }

    pub fn is_supported<'a, 'b, S>(&'a self, container_type: &Node) -> (UISupportQuality, Node)
    where
        S: UISupport,
        'a: 'b,
    {
        let mut ui_type: *const LilvNode = ptr::null_mut();
        let quality = UISupportQuality(unsafe {
            lilv_ui_is_supported(
                self.ui,
                Some(supported_func::<S>),
                container_type.node,
                &mut ui_type,
            )
        });
        (quality, ref_node(&self.world, ui_type))
    }

    pub fn get_bundle_uri(&self) -> Node {
        ref_node(&self.world, unsafe { lilv_ui_get_bundle_uri(self.ui) })
    }

    pub fn get_binary_uri(&self) -> Node {
        ref_node(&self.world, unsafe { lilv_ui_get_binary_uri(self.ui) })
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct UISupportQuality(pub u32);

impl Deref for UISupportQuality {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub trait UISupport {
    fn supported(container: &CStr, ui: &CStr) -> UISupportQuality;
}

unsafe extern "C" fn supported_func<S: UISupport>(
    container_type_uri: *const i8,
    ui_type_uri: *const i8,
) -> u32 {
    S::supported(
        &CStr::from_ptr(container_type_uri),
        &CStr::from_ptr(ui_type_uri),
    )
    .0
}
