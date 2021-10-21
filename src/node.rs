use crate::world::Life;
use lilv_sys as lib;
use std::ffi::CStr;
use std::ptr::NonNull;
use std::sync::Arc;

extern "C" {
    // Needed for Node::get_path as it actually returns
    // things allocated in Serd.
    fn serd_free(ptr: *mut std::os::raw::c_void);
}

unsafe impl Send for Node {}
unsafe impl Sync for Node {}

pub struct Node {
    pub(crate) inner: NonNull<lib::LilvNodeImpl>,
    pub(crate) borrowed: bool,
    pub(crate) life: Arc<Life>,
}

impl Node {
    /// Returns this value as a Turtle/SPARQL token.
    #[must_use]
    pub fn turtle_token(&self) -> String {
        let _life = self.life.inner.lock();
        let node = self.inner.as_ptr();

        unsafe {
            let raw = lib::lilv_node_get_turtle_token(node);
            let formatted = CStr::from_ptr(raw).to_string_lossy().into_owned();
            lib::lilv_free(raw.cast());
            formatted
        }
    }

    /// Returns whether the value is a URI (resource).
    #[must_use]
    pub fn is_uri(&self) -> bool {
        let _life = self.life.inner.lock();
        unsafe { lib::lilv_node_is_uri(self.inner.as_ptr()) }
    }

    /// Returns this value as a URI string.
    #[must_use]
    pub fn as_uri(&self) -> Option<&str> {
        if self.is_uri() {
            let _life = self.life.inner.lock();
            Some(unsafe {
                CStr::from_ptr(lib::lilv_node_as_uri(self.inner.as_ptr()))
                    .to_str()
                    .ok()?
            })
        } else {
            None
        }
    }

    /// Returns whether the value is a blank node (resource with no URI).
    #[must_use]
    pub fn is_blank(&self) -> bool {
        let _life = self.life.inner.lock();
        unsafe { lib::lilv_node_is_blank(self.inner.as_ptr()) }
    }

    /// Returns this value as a blank node identifier.
    #[must_use]
    pub fn as_blank(&self) -> Option<&str> {
        if self.is_blank() {
            let _life = self.life.inner.lock();
            Some(unsafe {
                CStr::from_ptr(lib::lilv_node_as_blank(self.inner.as_ptr()))
                    .to_str()
                    .ok()?
            })
        } else {
            None
        }
    }

    #[must_use]
    pub fn is_literal(&self) -> bool {
        let _life = self.life.inner.lock();
        unsafe { lib::lilv_node_is_literal(self.inner.as_ptr()) }
    }

    #[must_use]
    pub fn is_string(&self) -> bool {
        let _life = self.life.inner.lock();
        unsafe { lib::lilv_node_is_string(self.inner.as_ptr()) }
    }

    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        let _life = self.life.inner.lock();
        Some(unsafe {
            CStr::from_ptr(lib::lilv_node_as_string(self.inner.as_ptr()))
                .to_str()
                .ok()?
        })
    }

    #[must_use]
    pub fn get_path(&self) -> Option<(String, String)> {
        let node = self.inner.as_ptr();
        let mut raw_hostname = std::ptr::null_mut();
        let raw_path = NonNull::new(unsafe {
            let _life = self.life.inner.lock();
            lib::lilv_node_get_path(node, &mut raw_hostname)
        })?;

        unsafe {
            let path = CStr::from_ptr(raw_path.as_ptr())
                .to_string_lossy()
                .into_owned();
            let hostname = CStr::from_ptr(raw_hostname).to_string_lossy().into_owned();

            let _life = self.life.inner.lock();
            serd_free(raw_path.as_ptr().cast());
            serd_free(raw_hostname.cast());

            Some((path, hostname))
        }
    }

    #[must_use]
    pub fn is_float(&self) -> bool {
        let _life = self.life.inner.lock();
        unsafe { lib::lilv_node_is_float(self.inner.as_ptr()) }
    }

    #[must_use]
    pub fn as_float(&self) -> Option<f32> {
        if self.is_float() {
            let _life = self.life.inner.lock();
            Some(unsafe { lib::lilv_node_as_float(self.inner.as_ptr()) })
        } else {
            None
        }
    }

    #[must_use]
    pub fn is_int(&self) -> bool {
        let _life = self.life.inner.lock();
        unsafe { lib::lilv_node_is_int(self.inner.as_ptr()) }
    }

    #[must_use]
    pub fn as_int(&self) -> Option<i32> {
        if self.is_int() {
            let _life = self.life.inner.lock();
            Some(unsafe { lib::lilv_node_as_int(self.inner.as_ptr()) })
        } else {
            None
        }
    }

    #[must_use]
    pub fn is_bool(&self) -> bool {
        let _life = self.life.inner.lock();
        unsafe { lib::lilv_node_is_bool(self.inner.as_ptr()) }
    }

    #[must_use]
    pub fn as_bool(&self) -> Option<bool> {
        if self.is_bool() {
            let _life = self.life.inner.lock();
            Some(unsafe { lib::lilv_node_as_bool(self.inner.as_ptr()) })
        } else {
            None
        }
    }
}

impl Clone for Node {
    fn clone(&self) -> Self {
        let _life = self.life.inner.lock();
        Self {
            inner: NonNull::new(unsafe { lib::lilv_node_duplicate(self.inner.as_ptr()) }).unwrap(),
            borrowed: false,
            life: self.life.clone(),
        }
    }
}

impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("turtle_token", &self.turtle_token())
            .finish()
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        let _life = self.life.inner.lock();
        unsafe { lib::lilv_node_equals(self.inner.as_ptr(), other.inner.as_ptr()) }
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        if !self.borrowed {
            let _life = self.life.inner.lock();
            unsafe { lib::lilv_node_free(self.inner.as_ptr()) }
        }
    }
}

pub struct Nodes {
    pub(crate) inner: *const lib::LilvNodes,
    pub(crate) life: Arc<Life>,
}

impl Nodes {
    #[must_use]
    pub fn size(&self) -> usize {
        let _life = self.life.inner.lock();
        unsafe { lib::lilv_nodes_size(self.inner) as _ }
    }

    #[must_use]
    pub fn contains(&self, value: &Node) -> bool {
        let _life = self.life.inner.lock();
        let inner = self.inner;
        let value = value.inner.as_ptr();

        unsafe { lib::lilv_nodes_contains(inner, value) }
    }

    /// # Panics
    /// Panics if the merge is unsuccessful.
    #[must_use]
    pub fn merge(&self, other: &Self) -> Self {
        let _life = self.life.inner.lock();
        let a = self.inner;
        let b = other.inner;

        Nodes {
            inner: unsafe { lib::lilv_nodes_merge(a, b) },
            life: self.life.clone(),
        }
    }

    #[must_use]
    pub fn iter(&self) -> NodesIter<'_> {
        let _life = self.life.inner.lock();
        NodesIter {
            inner: unsafe { lib::lilv_nodes_begin(self.inner) },
            life: self.life.clone(),
            nodes: self,
        }
    }
}

impl std::fmt::Debug for Nodes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let nodes = self.iter().collect::<Vec<_>>();
        f.debug_struct("Nodes").field("nodes", &nodes).finish()
    }
}

pub struct NodesIter<'a> {
    inner: *mut lib::LilvIter,
    life: Arc<Life>,
    nodes: &'a Nodes,
}

impl<'a> Iterator for NodesIter<'a> {
    type Item = Node;

    fn next(&mut self) -> Option<Self::Item> {
        let _life = self.life.inner.lock();
        let node = unsafe { lib::lilv_nodes_get(self.nodes.inner, self.inner) } as *mut _;
        let next = Some({
            let ptr = NonNull::new(node)?;
            let world = self.life.clone();
            Node {
                inner: ptr,
                borrowed: true,
                life: world,
            }
        });
        self.inner = unsafe { lib::lilv_nodes_next(self.nodes.inner, self.inner) };
        next
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::World;

    #[test]
    fn test_null_nodes() {
        let world = World::new();
        let nodes = Nodes {
            inner: std::ptr::null(),
            life: world.life,
        };
        assert_eq!(nodes.size(), 0);
        for n in nodes.iter() {
            panic!("Should not have any nodes but found {}", n.turtle_token());
        }
    }
}
