use crate::world::new_node;
use crate::world::World;
use crate::Void;
use lilv_sys::*;
use std::ffi::CStr;
use std::ffi::CString;
use std::marker::PhantomData;
use std::ptr;
use std::rc::Rc;

#[link(name = "serd-0")]
extern "C" {
    fn serd_free(val: *mut Void);
}

pub struct Node<'a> {
    pub(crate) node: *mut LilvNode,
    pub(crate) world: Rc<World>,
    pub(crate) owned: bool,
    pub(crate) _phantom: PhantomData<(Value<'a>, fn() -> &'a ())>,
}

impl<'a> Clone for Node<'a> {
    fn clone(&self) -> Self {
        new_node(&self.world, unsafe { lilv_node_duplicate(self.node) })
    }
}

impl<'a> PartialEq for Node<'a> {
    fn eq(&self, other: &Self) -> bool {
        unsafe { lilv_node_equals(self.node, other.node) }
    }
}

impl<'a> Drop for Node<'a> {
    fn drop(&mut self) {
        if self.owned {
            let _lock = (*self.world).0.write().unwrap();
            unsafe { lilv_node_free(self.node) };
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Value<'a> {
    Uri(&'a CStr),
    Blank(&'a CStr),
    String(&'a CStr),
    Float(f32),
    Int(i32),
    Bool(bool),
}

impl<'a> Value<'a> {
    pub fn try_into_uri(self) -> Result<&'a CStr, Self> {
        if let Value::Uri(uri) = self {
            Ok(uri)
        } else {
            Err(self)
        }
    }

    pub fn try_into_blank(self) -> Result<&'a CStr, Self> {
        if let Value::Blank(blank) = self {
            Ok(blank)
        } else {
            Err(self)
        }
    }

    pub fn try_into_string(self) -> Result<&'a CStr, Self> {
        if let Value::String(string) = self {
            Ok(string)
        } else {
            Err(self)
        }
    }

    pub fn try_into_float(self) -> Result<f32, Self> {
        if let Value::Float(float) = self {
            Ok(float)
        } else {
            Err(self)
        }
    }

    pub fn try_into_int(self) -> Result<i32, Self> {
        if let Value::Int(int) = self {
            Ok(int)
        } else {
            Err(self)
        }
    }

    pub fn try_into_bool(self) -> Result<bool, Self> {
        if let Value::Bool(b) = self {
            Ok(b)
        } else {
            Err(self)
        }
    }

    pub fn into_uri(self) -> &'a CStr {
        self.try_into_uri().unwrap()
    }

    pub fn into_blank(self) -> &'a CStr {
        self.try_into_blank().unwrap()
    }

    pub fn into_string(self) -> &'a CStr {
        self.try_into_string().unwrap()
    }

    pub fn into_float(self) -> f32 {
        self.try_into_float().unwrap()
    }

    pub fn into_int(self) -> i32 {
        self.try_into_int().unwrap()
    }

    pub fn into_bool(self) -> bool {
        self.try_into_bool().unwrap()
    }
}

impl<'a> Node<'a> {
    pub fn get_turtle_token(&self) -> CString {
        unsafe {
            let token = lilv_node_get_turtle_token(self.node);
            let ret = CString::from(CStr::from_ptr(token));
            lilv_free(token as *mut Void);
            ret
        }
    }

    pub fn value(&self) -> Value<'a> {
        unsafe {
            if lilv_node_is_uri(self.node) {
                Value::Uri(CStr::from_ptr(lilv_node_as_uri(self.node)))
            } else if lilv_node_is_blank(self.node) {
                Value::Blank(CStr::from_ptr(lilv_node_as_blank(self.node)))
            } else if lilv_node_is_string(self.node) {
                Value::String(CStr::from_ptr(lilv_node_as_string(self.node)))
            } else if lilv_node_is_float(self.node) {
                Value::Float(lilv_node_as_float(self.node))
            } else if lilv_node_is_int(self.node) {
                Value::Int(lilv_node_as_int(self.node))
            } else if lilv_node_is_bool(self.node) {
                Value::Bool(lilv_node_as_bool(self.node))
            } else {
                unreachable!()
            }
        }
    }

    pub fn is_uri(&self) -> bool {
        unsafe { lilv_node_is_uri(self.node) }
    }

    pub fn is_blank(&self) -> bool {
        unsafe { lilv_node_is_blank(self.node) }
    }

    pub fn is_string(&self) -> bool {
        unsafe { lilv_node_is_string(self.node) }
    }

    pub fn is_float(&self) -> bool {
        unsafe { lilv_node_is_float(self.node) }
    }

    pub fn is_int(&self) -> bool {
        unsafe { lilv_node_is_int(self.node) }
    }

    pub fn is_bool(&self) -> bool {
        unsafe { lilv_node_is_bool(self.node) }
    }

    pub fn is_literal(&self) -> bool {
        unsafe { lilv_node_is_literal(self.node) }
    }

    pub fn get_path(&self, with_hostname: bool) -> Option<(CString, Option<CString>)> {
        if !self.is_uri() {
            return None;
        }

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

#[cfg(test)]
mod tests {
    use crate::node::Value;
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
            Value::Uri(CString::new("http://example.org").unwrap().as_ref()),
        );
    }

    #[test]
    fn float() {
        let w = world();
        let float = w.new_float(12.0);
        assert_eq!(float.value(), Value::Float(12.0));
        assert_eq!(float == w.new_float(12.0), true);
        assert_ne!(float == w.new_float(121.0), true);
    }

    #[test]
    fn int() {
        let w = world();
        let int = w.new_int(34);
        assert_eq!(int.value(), Value::Int(34));
        assert_eq!(int == w.new_int(34), true);
        assert_ne!(int == w.new_int(56), true);
    }

    #[test]
    fn any() {
        let w = world();
        let int = w.new_int(0);
        let float = w.new_float(0.0);
        assert_ne!(int == float, true);
        assert_ne!(int.value() == float.value(), true);
    }

    #[test]
    fn clone() {
        let w = world();
        let a = w.new_int(1337);
        let b = a.clone();
        assert_eq!(a == b, true);
    }
}
