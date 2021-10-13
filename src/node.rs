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
            let original = lib::lilv_node_get_turtle_token(node);
            let rusty = CStr::from_ptr(lib::lilv_node_get_turtle_token(self.inner.as_ptr()))
                .to_string_lossy()
                .into_owned();
            lib::lilv_free(original.cast());
            rusty
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
        let mut hostname = std::ptr::null_mut();
        let path = NonNull::new(unsafe {
            let _life = self.life.inner.lock();
            lib::lilv_node_get_path(node, &mut hostname)
        })?;

        unsafe {
            let rusty_path = CStr::from_ptr(path.as_ptr()).to_string_lossy().into_owned();
            let rusty_hostname = CStr::from_ptr(hostname).to_string_lossy().into_owned();

            let _life = self.life.inner.lock();
            serd_free(path.as_ptr().cast());
            serd_free(hostname.cast());

            Some((rusty_path, rusty_hostname))
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

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        let _life = self.life.inner.lock();
        unsafe { lib::lilv_node_equals(self.inner.as_ptr(), other.inner.as_ptr()) }
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        let _life = self.life.inner.lock();
        if !self.borrowed {
            unsafe { lib::lilv_node_free(self.inner.as_ptr()) }
        }
    }
}

pub struct Nodes {
    pub(crate) inner: NonNull<lib::LilvNodes>,
    pub(crate) life: Arc<Life>,
}

impl Nodes {
    #[must_use]
    pub(crate) fn new(inner: NonNull<lib::LilvNodes>, world: Arc<Life>) -> Self {
        Self { inner, life: world }
    }

    #[must_use]
    pub fn size(&self) -> usize {
        let _life = self.life.inner.lock();
        unsafe { lib::lilv_nodes_size(self.inner.as_ptr()) as _ }
    }

    #[must_use]
    pub fn contains(&self, value: &Node) -> bool {
        let _life = self.life.inner.lock();
        let inner = self.inner.as_ptr();
        let value = value.inner.as_ptr();

        unsafe { lib::lilv_nodes_contains(inner, value) }
    }

    /// # Panics
    /// Panics if the merge is unsuccessful.
    #[must_use]
    pub fn merge(&self, other: &Self) -> Self {
        let _life = self.life.inner.lock();
        let a = self.inner.as_ptr();
        let b = other.inner.as_ptr();

        Nodes {
            inner: NonNull::new(unsafe { lib::lilv_nodes_merge(a, b) }).unwrap(),
            life: self.life.clone(),
        }
    }

    #[must_use]
    pub fn iter(&self) -> NodesIter<'_> {
        let _life = self.life.inner.lock();
        NodesIter {
            inner: unsafe { lib::lilv_nodes_begin(self.inner.as_ptr()) },
            life: self.life.clone(),
            nodes: self,
        }
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
        let node = unsafe { lib::lilv_nodes_get(self.nodes.inner.as_ptr(), self.inner) } as *mut _;
        let next = Some({
            let ptr = NonNull::new(node)?;
            let world = self.life.clone();
            Node {
                inner: ptr,
                borrowed: true,
                life: world,
            }
        });
        self.inner = unsafe { lib::lilv_nodes_next(self.nodes.inner.as_ptr(), self.inner) };
        next
    }
}
