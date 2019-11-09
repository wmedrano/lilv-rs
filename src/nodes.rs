use crate::collection::Collection;
use crate::collection::Iter;
use crate::node::Node;
use crate::world::World;
use crate::Void;
use lilv_sys::*;
use std::marker::PhantomData;
use std::rc::Rc;

pub struct Nodes {
    pub(crate) nodes: *mut Void,
    pub(crate) world: Rc<World>,
    pub(crate) owned: bool,
}

impl Drop for Nodes {
    fn drop(&mut self) {
        if self.owned {
            unsafe { lilv_nodes_free(self.nodes) }
        }
    }
}

impl AsRef<*const Void> for Nodes {
    fn as_ref(&self) -> &*const Void {
        unsafe {
            &*(&self.nodes as *const *mut core::ffi::c_void as *const *const core::ffi::c_void)
        }
    }
}

impl<'a> Collection<'a> for Nodes
where
    Self: 'a,
{
    type Target = Node<'a>;

    unsafe fn get(&self, i: *mut Void) -> Self::Target {
        Node {
            node: lilv_nodes_get(self.nodes, i) as *mut LilvNodeImpl,
            world: self.world.clone(),
            owned: false,
            _phantom: PhantomData,
        }
    }
}

impl Nodes {
    pub fn contains(&self, value: &Node) -> bool {
        unsafe { lilv_nodes_contains(self.nodes, value.node) }
    }

    pub fn merge(&self, other: &Self) -> Self {
        Self {
            nodes: unsafe { lilv_nodes_merge(self.nodes, other.nodes) },
            world: self.world.clone(),
            owned: true,
        }
    }

    pub fn iter(&self) -> Iter<'_, Self> {
        Iter::new(self, lilv_nodes_begin, lilv_nodes_is_end, lilv_nodes_next)
    }

    pub fn size(&self) -> usize {
        unsafe { lilv_nodes_size(self.nodes) as usize }
    }
}
