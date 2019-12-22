use crate::node::Node;
use crate::nodes::Nodes;
use crate::plugin::Plugin;
use lilv_sys as lib;
use parking_lot::RwLock;
use std::ffi::CStr;
use std::ptr::NonNull;

pub struct UI<'a> {
    pub(crate) inner: RwLock<NonNull<lib::LilvUI>>,
    pub(crate) plugin: &'a Plugin,
}

impl<'a> UI<'a> {
    pub(crate) fn new_borrowed(ptr: NonNull<lib::LilvUI>, plugin: &'a Plugin) -> Self {
        Self {
            inner: RwLock::new(ptr),
            plugin,
        }
    }

    pub fn uri(&self) -> Node {
        let ui = self.inner.read().as_ptr();

        Node::new_borrowed(
            NonNull::new(unsafe { lib::lilv_ui_get_uri(ui) as _ }).unwrap(),
            self.plugin.world.clone(),
        )
    }

    pub fn classes(&self) -> Nodes {
        let ui = self.inner.read().as_ptr();

        Nodes::new_borrowed(
            NonNull::new(unsafe { lib::lilv_ui_get_classes(ui) as _ }).unwrap(),
            self.plugin.world.clone(),
        )
    }

    pub fn is_a(&self, class_uri: &Node) -> bool {
        let ui = self.inner.read().as_ptr();
        let class_uri = class_uri.inner.read().as_ptr();

        unsafe { lib::lilv_ui_is_a(ui, class_uri) }
    }

    pub fn is_supported<S>(
        &self,
        container_type: &Node,
        ui_type: Option<&mut Option<Node>>,
    ) -> UISupportQuality
    where
        S: UISupport,
    {
        let ui = self.inner.read().as_ptr();
        let container_type = container_type.inner.read().as_ptr();

        let mut ui_type_ptr = std::ptr::null();

        let quality = UISupportQuality(unsafe {
            lib::lilv_ui_is_supported(
                ui,
                Some(supported_func::<S>),
                container_type,
                ui_type
                    .as_ref()
                    .map(|_| &mut ui_type_ptr as _)
                    .unwrap_or(std::ptr::null_mut()),
            )
        });

        if let Some(ui_type) = ui_type {
            *ui_type = Some(Node::new_borrowed(
                NonNull::new(ui_type_ptr as _).unwrap(),
                self.plugin.world.clone(),
            ));
        }

        quality
    }

    pub fn bundle_uri(&self) -> Node {
        let ui = self.inner.read().as_ptr();

        Node::new_borrowed(
            NonNull::new(unsafe { lib::lilv_ui_get_bundle_uri(ui) as _ }).unwrap(),
            self.plugin.world.clone(),
        )
    }

    pub fn binary_uri(&self) -> Node {
        let ui = self.inner.read().as_ptr();

        Node::new_borrowed(
            NonNull::new(unsafe { lib::lilv_ui_get_binary_uri(ui) as _ }).unwrap(),
            self.plugin.world.clone(),
        )
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct UISupportQuality(pub u32);

pub trait UISupport {
    fn supported(container: &str, ui: &str) -> UISupportQuality;
}

unsafe extern "C" fn supported_func<S: UISupport>(
    container_type_uri: *const i8,
    ui_type_uri: *const i8,
) -> u32 {
    S::supported(
        &CStr::from_ptr(container_type_uri).to_str().unwrap(),
        &CStr::from_ptr(ui_type_uri).to_str().unwrap(),
    )
    .0
}
