use crate::node::Node;
use crate::nodes::Nodes;
use crate::plugin::Plugin;
use crate::scale_points::ScalePoints;
use lilv_sys as lib;
use parking_lot::RwLock;
use std::ptr::NonNull;

pub struct Port<'a> {
    pub(crate) inner: RwLock<NonNull<lib::LilvPort>>,
    pub(crate) plugin: &'a Plugin,
}

impl<'a> Port<'a> {
    pub(crate) fn new_borrowed(inner: NonNull<lib::LilvPort>, plugin: &'a Plugin) -> Self {
        Self {
            inner: RwLock::new(inner),
            plugin,
        }
    }

    pub fn node(&self) -> Node {
        let plugin = self.plugin.inner.read().as_ptr();
        let port = self.inner.read().as_ptr();

        Node::new_borrowed(
            NonNull::new(unsafe { lib::lilv_port_get_node(plugin, port) as _ }).unwrap(),
            self.plugin.world.clone(),
        )
    }

    pub fn value(&self, predicate: &Node) -> Option<Nodes> {
        let plugin = self.plugin.inner.read().as_ptr();
        let port = self.inner.read().as_ptr();
        let predicate = predicate.inner.read().as_ptr();

        Some(Nodes::new(
            NonNull::new(unsafe { lib::lilv_port_get_value(plugin, port, predicate) })?,
            self.plugin.world.clone(),
        ))
    }

    pub fn get(&self, predicate: &Node) -> Option<Node> {
        let plugin = self.plugin.inner.read().as_ptr();
        let port = self.inner.read().as_ptr();
        let predicate = predicate.inner.read().as_ptr();

        Some(Node::new(
            NonNull::new(unsafe { lib::lilv_port_get(plugin, port, predicate) })?,
            self.plugin.world.clone(),
        ))
    }

    pub fn properties(&self) -> Option<Nodes> {
        let plugin = self.plugin.inner.read().as_ptr();
        let port = self.inner.read().as_ptr();

        Some(Nodes::new(
            NonNull::new(unsafe { lib::lilv_port_get_properties(plugin, port) })?,
            self.plugin.world.clone(),
        ))
    }

    pub fn has_property(&self, property_uri: &Node) -> bool {
        let plugin = self.plugin.inner.read().as_ptr();
        let port = self.inner.read().as_ptr();
        let property_uri = property_uri.inner.read().as_ptr();

        unsafe { lib::lilv_port_has_property(plugin, port, property_uri) }
    }

    pub fn supports_event(&self, event_type: &Node) -> bool {
        let plugin = self.plugin.inner.read().as_ptr();
        let port = self.inner.read().as_ptr();
        let event_type = event_type.inner.read().as_ptr();

        unsafe { lib::lilv_port_supports_event(plugin, port, event_type) }
    }

    pub fn index(&self) -> usize {
        let plugin = self.plugin.inner.read().as_ptr();
        let port = self.inner.read().as_ptr();

        unsafe { lib::lilv_port_get_index(plugin, port) as _ }
    }

    pub fn symbol(&self) -> Node {
        let plugin = self.plugin.inner.read().as_ptr();
        let port = self.inner.read().as_ptr();

        Node::new_borrowed(
            NonNull::new(unsafe { lib::lilv_port_get_symbol(plugin, port) as _ }).unwrap(),
            self.plugin.world.clone(),
        )
    }

    pub fn name(&self) -> Option<Node> {
        let plugin = self.plugin.inner.read().as_ptr();
        let port = self.inner.read().as_ptr();

        Some(Node::new(
            NonNull::new(unsafe { lib::lilv_port_get_name(plugin, port) })?,
            self.plugin.world.clone(),
        ))
    }

    pub fn classes(&self) -> Nodes {
        let plugin = self.plugin.inner.read().as_ptr();
        let port = self.inner.read().as_ptr();

        Nodes::new(
            NonNull::new(unsafe { lib::lilv_port_get_classes(plugin, port) as _ }).unwrap(),
            self.plugin.world.clone(),
        )
    }

    pub fn is_a(&self, port_class: &Node) -> bool {
        let plugin = self.plugin.inner.read().as_ptr();
        let port = self.inner.read().as_ptr();
        let port_class = port_class.inner.read().as_ptr();

        unsafe { lib::lilv_port_is_a(plugin, port, port_class) }
    }

    pub fn range(
        &self,
        default: Option<&mut Option<Node>>,
        minimum: Option<&mut Option<Node>>,
        maximum: Option<&mut Option<Node>>,
    ) {
        let plugin = self.plugin.inner.read().as_ptr();
        let port = self.inner.read().as_ptr();

        let mut default_ptr = std::ptr::null_mut();
        let mut minimum_ptr = std::ptr::null_mut();
        let mut maximum_ptr = std::ptr::null_mut();

        unsafe {
            lib::lilv_port_get_range(
                plugin,
                port,
                default
                    .as_ref()
                    .map(|_| &mut default_ptr as _)
                    .unwrap_or(std::ptr::null_mut()),
                minimum
                    .as_ref()
                    .map(|_| &mut minimum_ptr as _)
                    .unwrap_or(std::ptr::null_mut()),
                maximum
                    .as_ref()
                    .map(|_| &mut maximum_ptr as _)
                    .unwrap_or(std::ptr::null_mut()),
            )
        }

        if let Some(default) = default {
            *default = Some(Node::new(
                NonNull::new(default_ptr).unwrap(),
                self.plugin.world.clone(),
            ));
        }

        if let Some(minimum) = minimum {
            *minimum = Some(Node::new(
                NonNull::new(minimum_ptr).unwrap(),
                self.plugin.world.clone(),
            ));
        }

        if let Some(maximum) = maximum {
            *maximum = Some(Node::new(
                NonNull::new(maximum_ptr).unwrap(),
                self.plugin.world.clone(),
            ));
        }
    }

    pub fn scale_points(&self) -> Option<ScalePoints> {
        let plugin = self.plugin.inner.read().as_ptr() as *const _;
        let port = self.inner.read().as_ptr() as *const _;

        Some(ScalePoints {
            inner: NonNull::new(unsafe { lib::lilv_port_get_scale_points(plugin, port) })?,
            port: self,
        })
    }
}
