use crate::world::new_node;
use crate::world::World;
use crate::Void;
use std::ffi::CStr;
use std::ffi::CString;
use std::marker::PhantomData;
use std::ptr;
use std::rc::Rc;

#[link(name = "lilv-0")]
extern "C" {
    fn lilv_node_duplicate(value: *const Void) -> *mut Void;
    fn lilv_node_equals(value: *const Void, other: *const Void) -> u8;
    fn lilv_node_get_turtle_token(value: *const Void) -> *mut i8;
    fn lilv_node_is_uri(value: *const Void) -> u8;
    fn lilv_node_as_uri(value: *const Void) -> *const i8;
    fn lilv_node_is_blank(value: *const Void) -> u8;
    fn lilv_node_as_blank(value: *const Void) -> *const i8;
    fn lilv_node_is_literal(value: *const Void) -> u8;
    fn lilv_node_is_string(value: *const Void) -> u8;
    fn lilv_node_as_string(value: *const Void) -> *const i8;
    fn lilv_node_get_path(value: *const Void, hostname: *mut *mut i8) -> *mut i8;
    fn lilv_node_is_float(value: *const Void) -> u8;
    fn lilv_node_as_float(value: *const Void) -> f32;
    fn lilv_node_is_int(value: *const Void) -> u8;
    fn lilv_node_as_int(value: *const Void) -> i32;
    fn lilv_node_is_bool(value: *const Void) -> u8;
    fn lilv_node_as_bool(value: *const Void) -> u8;
    fn lilv_node_free(val: *mut Void);
    fn lilv_free(val: *mut Void);
}

#[link(name = "serd-0")]
extern "C" {
    fn serd_free(val: *mut Void);
}

pub struct Node<'a, T> {
    pub(crate) node: *mut Void,
    pub(crate) world: Rc<World>,
    pub(crate) owned: bool,
    pub(crate) _phantom: PhantomData<(T, fn() -> &'a ())>,
}

impl<'a, T> Clone for Node<'a, T> {
    fn clone(&self) -> Self {
        new_node(&self.world, unsafe { lilv_node_duplicate(self.node) })
    }
}

impl<'a, T> PartialEq for Node<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        unsafe { lilv_node_equals(self.node, other.node) != 0 }
    }
}

impl<'a, T> Drop for Node<'a, T> {
    fn drop(&mut self) {
        if self.owned {
            let _lock = (*self.world).0.write().unwrap();
            unsafe { lilv_node_free(self.node) };
        }
    }
}

pub enum Any {}
pub enum Uri {}
pub enum Blank {}
pub enum Literal {}
pub enum String {}
pub enum Float {}
pub enum Int {}
pub enum Bool {}

pub trait NodeImpl<'a>: 'a {
    type Target: 'a;
    fn value(&'a self) -> Self::Target;
}

impl<'a, T> Node<'a, T> {
    pub fn as_any(self) -> Node<'a, Any> {
        self.convert::<Any>()
    }

    pub(crate) fn convert<U>(mut self) -> Node<'a, U> {
        let new_node = Node {
            node: self.node,
            world: self.world.clone(),
            owned: self.owned,
            _phantom: PhantomData,
        };
        self.owned = false;
        new_node
    }
}

impl<'a> Node<'a, Any> {
    pub fn get_turtle_token(&self) -> CString {
        unsafe {
            let token = lilv_node_get_turtle_token(self.node);
            let ret = CString::from(CStr::from_ptr(token));
            lilv_free(token as *mut Void);
            ret
        }
    }

    pub fn as_uri(self) -> Result<Node<'a, Uri>, Self> {
        unsafe {
            if lilv_node_is_uri(self.node) != 0 {
                Ok(self.convert())
            } else {
                Err(self)
            }
        }
    }

    pub fn as_blank(self) -> Result<Node<'a, Blank>, Self> {
        unsafe {
            if lilv_node_is_blank(self.node) != 0 {
                Ok(self.convert())
            } else {
                Err(self)
            }
        }
    }

    pub fn as_literal(self) -> Result<Node<'a, Literal>, Self> {
        unsafe {
            if lilv_node_is_literal(self.node) != 0 {
                Ok(self.convert())
            } else {
                Err(self)
            }
        }
    }

    pub fn as_string(self) -> Result<Node<'a, crate::node::String>, Self> {
        unsafe {
            if lilv_node_is_string(self.node) != 0 {
                Ok(self.convert())
            } else {
                Err(self)
            }
        }
    }

    pub fn as_float(self) -> Result<Node<'a, Float>, Self> {
        unsafe {
            if lilv_node_is_float(self.node) != 0 {
                Ok(self.convert())
            } else {
                Err(self)
            }
        }
    }

    pub fn as_int(self) -> Result<Node<'a, Int>, Self> {
        unsafe {
            if lilv_node_is_int(self.node) != 0 {
                Ok(self.convert())
            } else {
                Err(self)
            }
        }
    }

    pub fn as_bool(self) -> Result<Node<'a, Bool>, Self> {
        unsafe {
            if lilv_node_is_bool(self.node) != 0 {
                Ok(self.convert())
            } else {
                Err(self)
            }
        }
    }
}

impl<'a> NodeImpl<'a> for Node<'a, Uri> {
    type Target = &'a CStr;

    fn value(&self) -> Self::Target {
        unsafe { CStr::from_ptr(lilv_node_as_uri(self.node)) }
    }
}

impl<'a> NodeImpl<'a> for Node<'a, Blank> {
    type Target = &'a CStr;

    fn value(&self) -> Self::Target {
        unsafe { CStr::from_ptr(lilv_node_as_blank(self.node)) }
    }
}

impl<'a> Node<'a, Literal> {
    pub fn as_string(self) -> Result<Node<'a, crate::node::String>, Self> {
        match self.as_any().as_string() {
            Ok(node) => Ok(node),
            Err(node) => Err(node.convert()),
        }
    }

    pub fn as_float(self) -> Result<Node<'a, Float>, Self> {
        match self.as_any().as_float() {
            Ok(node) => Ok(node),
            Err(node) => Err(node.convert()),
        }
    }

    pub fn as_int(self) -> Result<Node<'a, Int>, Self> {
        match self.as_any().as_int() {
            Ok(node) => Ok(node),
            Err(node) => Err(node.convert()),
        }
    }

    pub fn as_bool(self) -> Result<Node<'a, Bool>, Self> {
        match self.as_any().as_bool() {
            Ok(node) => Ok(node),
            Err(node) => Err(node.convert()),
        }
    }
}

impl<'a> Node<'a, Uri> {
    pub fn get_path(&self, with_hostname: bool) -> Option<(CString, Option<CString>)> {
        let mut hostname = ptr::null_mut();

        let path = unsafe {
            lilv_node_get_path(
                self.node,
                if with_hostname {
                    &mut hostname
                } else {
                    ptr::null_mut()
                },
            )
        };

        if path.is_null() {
            None
        } else {
            unsafe {
                let ret_path = CString::from(CStr::from_ptr(path));
                lilv_free(path as *mut Void);

                let ret_hostname = if with_hostname & !hostname.is_null() {
                    let ret_hostname = CString::from(CStr::from_ptr(hostname));
                    serd_free(hostname as *mut Void);
                    Some(ret_hostname)
                } else {
                    None
                };

                Some((ret_path, ret_hostname))
            }
        }
    }
}

impl<'a> NodeImpl<'a> for Node<'a, crate::node::String> {
    type Target = &'a CStr;

    fn value(&self) -> Self::Target {
        unsafe { CStr::from_ptr(lilv_node_as_string(self.node)) }
    }
}

impl<'a> NodeImpl<'a> for Node<'a, Float> {
    type Target = f32;

    fn value(&self) -> Self::Target {
        unsafe { lilv_node_as_float(self.node) }
    }
}

impl<'a> NodeImpl<'a> for Node<'a, Int> {
    type Target = i32;

    fn value(&self) -> Self::Target {
        unsafe { lilv_node_as_int(self.node) }
    }
}

impl<'a> NodeImpl<'a> for Node<'a, Bool> {
    type Target = bool;

    fn value(&self) -> Self::Target {
        unsafe { lilv_node_as_bool(self.node) != 0 }
    }
}

#[cfg(test)]
mod tests {
    use crate::node::NodeImpl;
    use crate::world::World;
    use crate::world::WorldImpl;
    use std::ffi::CString;
    use std::rc::Rc;

    fn world() -> Rc<World> {
        World::new().unwrap()
    }

    #[test]
    fn uri() {
        let w = world();
        let uri = w.new_uri(&CString::new("http://example.org").unwrap());
        assert_eq!(
            uri.value(),
            CString::new("http://example.org").unwrap().as_ref()
        );
    }

    #[test]
    fn float() {
        let w = world();
        let float = w.new_float(12.0);
        assert_eq!(float.value(), 12.0);
        assert_eq!(float == w.new_float(12.0), true);
        assert_ne!(float == w.new_float(121.0), true);
    }

    #[test]
    fn int() {
        let w = world();
        let int = w.new_int(34);
        assert_eq!(int.value(), 34);
        assert_eq!(int == w.new_int(34), true);
        assert_ne!(int == w.new_int(56), true);
    }

    #[test]
    fn any() {
        let w = world();
        let int = w.new_int(0);
        let float = w.new_float(0.0);
        assert_ne!(int.as_any() == float.as_any(), true);
    }

    #[test]
    fn clone() {
        let w = world();
        let a = w.new_int(1337);
        let b = a.clone();
        assert_eq!(a == b, true);
    }
}
