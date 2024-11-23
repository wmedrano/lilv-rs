use lilv_sys as lib;
use std::{ptr::NonNull, sync::Arc};

use crate::{node::Node, world::Life};

pub struct State {
    pub(crate) world: Arc<Life>,
    pub(crate) inner: NonNull<lib::LilvState>,
}

impl State {
    pub fn plugin_uri(&self) -> Node {
        let node_ptr = unsafe { lib::lilv_state_get_plugin_uri(self.inner.as_ptr()) }.cast_mut();
        let node = Node {
            inner: NonNull::new(node_ptr).unwrap(),
            borrowed: true,
            life: self.world.clone(),
        };
        node.clone()
    }

    pub fn as_ptr(&self) -> *mut lib::LilvState {
        self.inner.as_ptr()
    }
}

impl Drop for State {
    fn drop(&mut self) {
        unsafe { lib::lilv_state_free(self.inner.as_ptr()) }
    }
}

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        let res = unsafe { lib::lilv_state_equals(self.inner.as_ptr(), other.inner.as_ptr()) };
        res
    }
}
