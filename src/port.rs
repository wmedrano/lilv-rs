use crate::node::{Node, Nodes};
use crate::plugin::Plugin;
use lilv_sys as lib;
use std::fmt::Debug;
use std::ptr::NonNull;

#[derive(Clone)]
pub struct Port {
    pub(crate) inner: NonNull<lib::LilvPort>,
    pub(crate) plugin: Plugin,
}

impl Port {
    /// # Panics
    /// Panics if the node could not be obtained.
    #[must_use]
    pub fn node(&self) -> Node {
        let _life = self.plugin.life.inner.lock();
        let plugin = self.plugin.inner.as_ptr();
        let port = self.inner.as_ptr();

        {
            let ptr = NonNull::new(unsafe { lib::lilv_port_get_node(plugin, port) as _ }).unwrap();
            let world = self.plugin.life.clone();
            Node {
                inner: ptr,
                borrowed: true,
                life: world,
            }
        }
    }

    #[must_use]
    pub fn value(&self, predicate: &Node) -> Nodes {
        let _life = self.plugin.life.inner.lock();
        let plugin = self.plugin.inner.as_ptr();
        let port = self.inner.as_ptr();
        let predicate = predicate.inner.as_ptr();
        let inner = unsafe { lib::lilv_port_get_value(plugin, port, predicate) };
        let world = self.plugin.life.clone();
        Nodes { inner, life: world }
    }

    #[must_use]
    pub fn get(&self, predicate: &Node) -> Option<Node> {
        let _life = self.plugin.life.inner.lock();
        let plugin = self.plugin.inner.as_ptr();
        let port = self.inner.as_ptr();
        let predicate = predicate.inner.as_ptr();

        Some({
            let ptr = NonNull::new(unsafe { lib::lilv_port_get(plugin, port, predicate) })?;
            let world = self.plugin.life.clone();
            Node {
                inner: ptr,
                borrowed: false,
                life: world,
            }
        })
    }

    #[must_use]
    pub fn properties(&self) -> Nodes {
        let _life = self.plugin.life.inner.lock();
        let plugin = self.plugin.inner.as_ptr();
        let port = self.inner.as_ptr();
        let inner = unsafe { lib::lilv_port_get_properties(plugin, port) };
        let world = self.plugin.life.clone();
        Nodes { inner, life: world }
    }

    #[must_use]
    pub fn has_property(&self, property_uri: &Node) -> bool {
        let _life = self.plugin.life.inner.lock();
        let plugin = self.plugin.inner.as_ptr();
        let port = self.inner.as_ptr();
        let property_uri = property_uri.inner.as_ptr();

        unsafe { lib::lilv_port_has_property(plugin, port, property_uri) }
    }

    #[must_use]
    pub fn supports_event(&self, event_type: &Node) -> bool {
        let _life = self.plugin.life.inner.lock();
        let plugin = self.plugin.inner.as_ptr();
        let port = self.inner.as_ptr();
        let event_type = event_type.inner.as_ptr();

        unsafe { lib::lilv_port_supports_event(plugin, port, event_type) }
    }

    #[must_use]
    pub fn index(&self) -> usize {
        let _life = self.plugin.life.inner.lock();
        let plugin = self.plugin.inner.as_ptr();
        let port = self.inner.as_ptr();

        unsafe { lib::lilv_port_get_index(plugin, port) as _ }
    }

    #[must_use]
    pub fn symbol(&self) -> Option<Node> {
        let _life = self.plugin.life.inner.lock();
        let plugin = self.plugin.inner.as_ptr();
        let port = self.inner.as_ptr();

        {
            let ptr = NonNull::new(unsafe { lib::lilv_port_get_symbol(plugin, port) as _ })?;
            let world = self.plugin.life.clone();
            Node {
                inner: ptr,
                borrowed: true,
                life: world,
            }
        }
        .into()
    }

    #[must_use]
    pub fn name(&self) -> Option<Node> {
        let _life = self.plugin.life.inner.lock();
        let plugin = self.plugin.inner.as_ptr();
        let port = self.inner.as_ptr();

        Some({
            let ptr = NonNull::new(unsafe { lib::lilv_port_get_name(plugin, port) })?;
            let world = self.plugin.life.clone();
            Node {
                inner: ptr,
                borrowed: false,
                life: world,
            }
        })
    }

    #[must_use]
    pub fn classes(&self) -> Nodes {
        let _life = self.plugin.life.inner.lock();
        let plugin = self.plugin.inner.as_ptr();
        let port = self.inner.as_ptr();
        let inner = unsafe { lib::lilv_port_get_classes(plugin, port) };
        let world = self.plugin.life.clone();
        Nodes { inner, life: world }
    }

    #[must_use]
    pub fn is_a(&self, port_class: &Node) -> bool {
        let _life = self.plugin.life.inner.lock();
        let plugin = self.plugin.inner.as_ptr();
        let port = self.inner.as_ptr();
        let port_class = port_class.inner.as_ptr();

        unsafe { lib::lilv_port_is_a(plugin, port, port_class) }
    }

    /// # Panics
    /// Panics if the range could not be obtained.
    #[must_use]
    pub fn range(&self) -> Range {
        let _life = self.plugin.life.inner.lock();
        let plugin = self.plugin.inner.as_ptr();
        let port = self.inner.as_ptr();

        let mut default_ptr: *mut lib::LilvNodeImpl = std::ptr::null_mut();
        let mut minimum_ptr: *mut lib::LilvNodeImpl = std::ptr::null_mut();
        let mut maximum_ptr: *mut lib::LilvNodeImpl = std::ptr::null_mut();

        unsafe {
            lib::lilv_port_get_range(
                plugin,
                port,
                &mut default_ptr,
                &mut minimum_ptr,
                &mut maximum_ptr,
            );
        };
        let ptr_to_node = |ptr: *mut lib::LilvNodeImpl| -> Option<Node> {
            let ptr = NonNull::new(ptr.cast())?;
            let world = self.plugin.life.clone();
            Some(Node {
                inner: ptr,
                borrowed: false,
                life: world,
            })
        };
        Range {
            default: ptr_to_node(default_ptr),
            minimum: ptr_to_node(minimum_ptr),
            maximum: ptr_to_node(maximum_ptr),
        }
    }

    #[must_use]
    pub fn scale_points(&self) -> ScalePoints {
        let _life = self.plugin.life.inner.lock();
        let plugin = self.plugin.inner.as_ptr() as *const _;
        let port = self.inner.as_ptr() as *const _;

        ScalePoints {
            inner: unsafe { lib::lilv_port_get_scale_points(plugin, port) },
            port: self.clone(),
        }
    }
}

impl Debug for Port {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Port")
            .field("name", &self.name())
            .field("symbol", &self.symbol())
            .field("classes", &self.classes())
            .field("range", &self.range())
            .field("properties", &self.properties())
            .finish()
    }
}

unsafe impl Send for ScalePoint {}
unsafe impl Sync for ScalePoint {}

#[derive(Clone)]
pub struct ScalePoint {
    pub(crate) inner: NonNull<lib::LilvScalePoint>,
    pub(crate) port: Port,
}

impl ScalePoint {
    /// # Panics
    /// Panics if the node for the value could not be obtained.
    #[must_use]
    pub fn label(&self) -> Node {
        let _life = self.port.plugin.life.inner.lock();
        let inner = self.inner.as_ptr();

        {
            let ptr = NonNull::new(unsafe { lib::lilv_scale_point_get_label(inner) as _ }).unwrap();
            let world = self.port.plugin.life.clone();
            Node {
                inner: ptr,
                borrowed: true,
                life: world,
            }
        }
    }

    /// # Panics
    /// Panics if the node for the value could not be obtained.
    #[must_use]
    pub fn value(&self) -> Node {
        let _life = self.port.plugin.life.inner.lock();
        let inner = self.inner.as_ptr();

        {
            let ptr = NonNull::new(unsafe { lib::lilv_scale_point_get_value(inner) as _ }).unwrap();
            let world = self.port.plugin.life.clone();
            Node {
                inner: ptr,
                borrowed: true,
                life: world,
            }
        }
    }
}

impl Debug for ScalePoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScalePoint")
            .field("label", &self.label())
            .field("value", &self.value())
            .finish()
    }
}

#[derive(Clone)]
pub struct ScalePoints {
    pub(crate) inner: *const lib::LilvScalePoints,
    pub(crate) port: Port,
}

impl ScalePoints {
    #[must_use]
    pub fn count(&self) -> usize {
        let _life = self.port.plugin.life.inner.lock();
        let size: u32 = unsafe { lib::lilv_scale_points_size(self.inner) };
        size as usize
    }

    #[must_use]
    pub fn iter(&self) -> ScalePointsIter {
        let _life = self.port.plugin.life.inner.lock();
        ScalePointsIter {
            inner: self.clone(),
            iter: unsafe { lib::lilv_scale_points_begin(self.inner) },
        }
    }
}

impl Debug for ScalePoints {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pts = self.iter().collect::<Vec<_>>();
        f.debug_struct("ScalePoints")
            .field("scale_points", &pts)
            .finish()
    }
}

impl IntoIterator for ScalePoints {
    type Item = ScalePoint;

    type IntoIter = ScalePointsIter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[derive(Clone)]
pub struct ScalePointsIter {
    inner: ScalePoints,
    iter: *mut lib::LilvIter,
}

impl Iterator for ScalePointsIter {
    type Item = ScalePoint;

    fn next(&mut self) -> Option<ScalePoint> {
        let _life = self.inner.port.plugin.life.inner.lock();
        let next_ptr =
            unsafe { lib::lilv_scale_points_get(self.inner.inner, self.iter.cast()) } as *mut _;
        let next = Some(ScalePoint {
            inner: NonNull::new(next_ptr)?,
            port: self.inner.port.clone(),
        });
        self.iter = unsafe { lib::lilv_scale_points_next(self.inner.inner, self.iter) };
        next
    }
}

/// Describe the ranges of the port if possible.
#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug, PartialEq)]
pub struct Range {
    /// The default value of the port.
    pub default: Option<Node>,
    /// The minimum value of the port.
    pub minimum: Option<Node>,
    /// The maximum value of the port.
    pub maximum: Option<Node>,
}

/// Describes the range of the ports of a plugin.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct FloatRanges {
    /// The default value of the port.
    pub default: f32,
    /// The minimum value of the port.
    pub min: f32,
    /// The maximum value of the port.
    pub max: f32,
}
