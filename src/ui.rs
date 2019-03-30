use crate::node::Node;
use crate::node::Uri;
use crate::nodes::Nodes;
use crate::world::ref_node;
use crate::world::World;
use crate::Void;
use std::ffi::CStr;
use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr;
use std::rc::Rc;

type UISupportedFunc = unsafe extern "C" fn(*const i8, *const i8) -> u32;

#[link(name = "lilv-0")]
extern "C" {
    fn lilv_ui_get_uri(ui: *const Void) -> *const Void;
    fn lilv_ui_get_classes(ui: *const Void) -> *const Void;
    fn lilv_ui_is_a(ui: *const Void, class_uri: *const Void) -> u8;
    fn lilv_ui_is_supported(
        ui: *const Void,
        supported_func: UISupportedFunc,
        container_type: *const Void,
        ui_type: *mut *const Void,
    ) -> u32;
    fn lilv_ui_get_bundle_uri(ui: *const Void) -> *const Void;
    fn lilv_ui_get_binary_uri(ui: *const Void) -> *const Void;
}

pub struct UI {
    pub(crate) ui: *mut Void,
    pub(crate) world: Rc<World>,
}

impl UI {
    pub fn get_uri(&self) -> Node<Uri> {
        ref_node(&self.world, unsafe { lilv_ui_get_uri(self.ui) })
    }

    pub fn get_classes(&self) -> Nodes<Uri> {
        Nodes {
            nodes: unsafe { lilv_ui_get_classes(self.ui) as *mut Void },
            world: self.world.clone(),
            owned: false,
            _phantom: PhantomData,
        }
    }

    pub fn is_a(&self, class_uri: &Node<Uri>) -> bool {
        unsafe { lilv_ui_is_a(self.ui, class_uri.node) != 0 }
    }

    pub fn is_supported<'a, 'b, S>(
        &'a self,
        container_type: &Node<Uri>,
    ) -> (UISupportQuality, Node<'b, Uri>)
    where
        S: UISupport,
        'a: 'b,
    {
        let mut ui_type: *const Void = ptr::null_mut();
        let quality = UISupportQuality(unsafe {
            lilv_ui_is_supported(
                self.ui,
                supported_func::<S>,
                container_type.node,
                &mut ui_type,
            )
        });
        (quality, ref_node(&self.world, ui_type))
    }

    pub fn get_bundle_uri(&self) -> Node<Uri> {
        ref_node(&self.world, unsafe { lilv_ui_get_bundle_uri(self.ui) })
    }

    pub fn get_binary_uri(&self) -> Node<Uri> {
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
