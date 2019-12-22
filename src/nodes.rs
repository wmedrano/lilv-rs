use crate::collection::*;
use crate::node::Node;
use crate::world::InnerWorld;
use lilv_sys as lib;
use std::ptr::NonNull;
use std::sync::Arc;

pub type Nodes = Collection<lib::LilvNodes, lib::LilvNode, Node>;

impl Nodes {
    pub fn size(&self) -> usize {
        unsafe { lib::lilv_nodes_size(self.inner.read().as_ptr()) as _ }
    }

    pub fn contains(&self, value: &Node) -> bool {
        let inner = self.inner.read().as_ptr();
        let value = value.inner.read().as_ptr();

        unsafe { lib::lilv_nodes_contains(inner, value) }
    }

    pub fn merge(&self, other: &Self) -> Self {
        let a = self.inner.read().as_ptr();
        let b = other.inner.read().as_ptr();

        Self::new(
            NonNull::new(unsafe { lib::lilv_nodes_merge(a, b) }).unwrap(),
            self.owner.clone(),
        )
    }
}

impl CollectionTrait for Nodes {
    type Inner = lib::LilvNodes;
    type InnerTarget = lib::LilvNode;
    type Target = Node;
    type Owner = Arc<InnerWorld>;

    unsafe fn inner(&self) -> *const Self::Inner {
        self.inner.read().as_ptr()
    }

    fn begin_fn() -> BeginFn<Self> {
        lib::lilv_nodes_begin
    }

    fn is_end_fn() -> IsEndFn<Self> {
        lib::lilv_nodes_is_end
    }

    fn get_fn() -> GetFn<Self> {
        lib::lilv_nodes_get
    }

    fn next_fn() -> NextFn<Self> {
        lib::lilv_nodes_next
    }

    fn free_fn() -> FreeFn<Self> {
        lib::lilv_nodes_free
    }

    fn get(&self, i: *mut lib::LilvIter) -> Self::Target {
        Node::new_borrowed(
            NonNull::new(unsafe { Self::get_fn()(self.inner(), i) as _ }).unwrap(),
            self.owner.clone(),
        )
    }
}
