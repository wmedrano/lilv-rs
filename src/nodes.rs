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

    pub fn iter(&self) -> NodesIter<'_> {
        NodesIter {
            nodes: self,
            iter: unsafe { lilv_nodes_begin(self.nodes) },
        }
    }

    pub fn len(&self) -> usize {
        unsafe { lilv_nodes_size(self.nodes) as usize }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub struct NodesIter<'a> {
    nodes: &'a Nodes,
    iter: *mut LilvIter,
}

impl<'a> Iterator for NodesIter<'a> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Node<'a>> {
        let ptr = unsafe { lilv_nodes_get(self.nodes.nodes, self.iter) };
        if ptr.is_null() {
            None
        } else {
            self.iter = unsafe { lilv_nodes_next(self.nodes.nodes, self.iter) };
            Some(Node {
                node: ptr as *mut LilvNode,
                world: self.nodes.world.clone(),
                owned: false,
                _phantom: PhantomData,
            })
        }
    }
}
