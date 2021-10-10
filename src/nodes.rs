use crate::node::Node;
use crate::world::InnerWorld;
use lilv_sys as lib;
use std::ptr::NonNull;
use std::sync::Arc;

pub struct Nodes {
    pub(crate) inner: NonNull<lib::LilvNodes>,
    pub(crate) world: Arc<InnerWorld>,
}

impl Nodes {
    pub(crate) fn new(inner: NonNull<lib::LilvNodes>, world: Arc<InnerWorld>) -> Self {
        Self { inner, world }
    }

    pub fn size(&self) -> usize {
        unsafe { lib::lilv_nodes_size(self.inner.as_ptr()) as _ }
    }

    pub fn contains(&self, value: &Node) -> bool {
        let inner = self.inner.as_ptr();
        let value = value.inner.read().as_ptr();

        unsafe { lib::lilv_nodes_contains(inner, value) }
    }

    pub fn merge(&self, other: &Self) -> Self {
        let a = self.inner.as_ptr();
        let b = other.inner.as_ptr();

        Nodes {
            inner: NonNull::new(unsafe { lib::lilv_nodes_merge(a, b) }).unwrap(),
            world: self.world.clone(),
        }
    }

    pub fn iter(&self) -> NodesIter<'_> {
        NodesIter {
            inner: unsafe { lib::lilv_nodes_begin(self.inner.as_ptr()) },
            world: self.world.clone(),
            nodes: self,
        }
    }
}

pub struct NodesIter<'a> {
    inner: *mut lib::LilvIter,
    world: Arc<InnerWorld>,
    nodes: &'a Nodes,
}

impl<'a> Iterator for NodesIter<'a> {
    type Item = Node;

    fn next(&mut self) -> Option<Self::Item> {
        let node = unsafe { lib::lilv_nodes_get(self.nodes.inner.as_ptr(), self.inner) } as *mut _;
        let next = Some(Node::new_borrowed(NonNull::new(node)?, self.world.clone()));
        self.inner = unsafe { lib::lilv_nodes_next(self.nodes.inner.as_ptr(), self.inner) };
        next
    }
}
