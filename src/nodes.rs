use crate::collection::Collection;
use crate::collection::Iter;
use crate::node::Node;
use crate::world::World;
use crate::Void;
use std::marker::PhantomData;
use std::mem;
use std::rc::Rc;

#[link(name = "lilv-0")]
extern "C" {
    fn lilv_nodes_free(nodes: *mut Void);
    fn lilv_nodes_size(nodes: *const Void) -> u32;
    fn lilv_nodes_get(nodes: *const Void, i: *mut Void) -> *const Void;
    fn lilv_nodes_begin(nodes: *const Void) -> *mut Void;
    fn lilv_nodes_next(nodes: *const Void, i: *mut Void) -> *mut Void;
    fn lilv_nodes_is_end(nodes: *const Void, i: *mut Void) -> u8;
    fn lilv_nodes_contains(nodes: *const Void, value: *const Void) -> u8;
    fn lilv_nodes_merge(a: *const Void, b: *const Void) -> *mut Void;

// Unnecessary? `nodes.iter().nth(0)`
// fn lilv_nodes_get_first(nodes: *const Void) -> *const Void;
}

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
        unsafe { mem::transmute(&self.nodes) }
    }
}

impl<'a> Collection<'a> for Nodes
where
    Self: 'a,
{
    type Target = Node<'a>;

    fn get(&self, i: *mut Void) -> Self::Target {
        Node {
            node: unsafe { lilv_nodes_get(self.nodes, i) as *mut Void },
            world: self.world.clone(),
            owned: false,
            _phantom: PhantomData,
        }
    }
}

impl Nodes {
    pub fn contains(&self, value: &Node) -> bool {
        unsafe { lilv_nodes_contains(self.nodes, value.node) != 0 }
    }

    pub fn merge(&self, other: &Self) -> Self {
        Self {
            nodes: unsafe { lilv_nodes_merge(self.nodes, other.nodes) },
            world: self.world.clone(),
            owned: true,
        }
    }

    pub fn iter<'a>(&'a self) -> Iter<'a, Self> {
        Iter::new(self, lilv_nodes_begin, lilv_nodes_is_end, lilv_nodes_next)
    }

    pub fn size(&self) -> usize {
        unsafe { lilv_nodes_size(self.nodes) as usize }
    }
}
