use crate::node::{Node, Nodes};
use crate::plugin::Plugin;
use lilv_sys as lib;
use std::ptr::NonNull;

pub struct Port<'a> {
    pub(crate) inner: NonNull<lib::LilvPort>,
    pub(crate) plugin: &'a Plugin,
}

impl<'a> Port<'a> {
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
    pub fn value(&self, predicate: &Node) -> Option<Nodes> {
        let _life = self.plugin.life.inner.lock();
        let plugin = self.plugin.inner.as_ptr();
        let port = self.inner.as_ptr();
        let predicate = predicate.inner.as_ptr();

        Some({
            let inner = NonNull::new(unsafe { lib::lilv_port_get_value(plugin, port, predicate) })?;
            let world = self.plugin.life.clone();
            Nodes { inner, life: world }
        })
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
    pub fn properties(&self) -> Option<Nodes> {
        let _life = self.plugin.life.inner.lock();
        let plugin = self.plugin.inner.as_ptr();
        let port = self.inner.as_ptr();

        Some({
            let inner = NonNull::new(unsafe { lib::lilv_port_get_properties(plugin, port) })?;
            let world = self.plugin.life.clone();
            Nodes { inner, life: world }
        })
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
    pub fn classes(&self) -> Option<Nodes> {
        let _life = self.plugin.life.inner.lock();
        let plugin = self.plugin.inner.as_ptr();
        let port = self.inner.as_ptr();

        {
            let inner = NonNull::new(unsafe { lib::lilv_port_get_classes(plugin, port) as _ })?;
            let world = self.plugin.life.clone();
            Nodes { inner, life: world }
        }
        .into()
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
    pub fn range(&self) -> PortRange {
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
        let ptr_to_node = |ptr| -> Option<Node> {
            let ptr = NonNull::new(ptr as _)?;
            let world = self.plugin.life.clone();
            Some(Node {
                inner: ptr,
                borrowed: false,
                life: world,
            })
        };
        PortRange {
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
            port: self,
        }
    }
}

unsafe impl<'a> Send for ScalePoint<'a> {}
unsafe impl<'a> Sync for ScalePoint<'a> {}

pub struct ScalePoint<'a> {
    pub(crate) inner: NonNull<lib::LilvScalePoint>,
    pub(crate) port: &'a Port<'a>,
}

impl<'a> ScalePoint<'a> {
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

pub struct ScalePoints<'a> {
    pub(crate) inner: *const lib::LilvScalePoints,
    pub(crate) port: &'a Port<'a>,
}

impl<'a> ScalePoints<'a> {
    #[must_use]
    pub fn size(&self) -> usize {
        let _life = self.port.plugin.life.inner.lock();
        let size: u32 = unsafe { lib::lilv_scale_points_size(self.inner) };
        size as usize
    }

    #[must_use]
    pub fn iter(&self) -> ScalePointsIter<'_> {
        let _life = self.port.plugin.life.inner.lock();
        ScalePointsIter {
            inner: self,
            iter: unsafe { lib::lilv_scale_points_begin(self.inner) },
        }
    }
}

pub struct ScalePointsIter<'a> {
    inner: &'a ScalePoints<'a>,
    iter: *mut lib::LilvIter,
}

impl<'a> Iterator for ScalePointsIter<'a> {
    type Item = ScalePoint<'a>;

    fn next(&mut self) -> Option<ScalePoint<'a>> {
        let _life = self.inner.port.plugin.life.inner.lock();
        let next_ptr =
            unsafe { lib::lilv_scale_points_get(self.inner.inner, self.iter.cast()) } as *mut _;
        let next = Some(ScalePoint {
            inner: NonNull::new(next_ptr)?,
            port: self.inner.port,
        });
        self.iter = unsafe { lib::lilv_scale_points_next(self.inner.inner, self.iter) };
        next
    }
}

/// Describe the ranges of the port if possible.
pub struct PortRange {
    /// The default value of the port.
    pub default: Option<Node>,
    /// The minimum value of the port.
    pub minimum: Option<Node>,
    /// The maximum value of the port.
    pub maximum: Option<Node>,
}
