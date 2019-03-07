use crate::collection::Collection;
use crate::collection::Iter;
use crate::node::Node;
use crate::node::Uri;
use crate::plugin::Plugin;
use crate::world::World;
use crate::Void;
use std::rc::Rc;

#[link(name = "lilv-0")]
extern "C" {
    fn lilv_plugins_get_by_uri(plugins: *const Void, uri: *const Void) -> *const Void;
    fn lilv_plugins_get(plugins: *const Void, i: *mut Void) -> *const Void;
    fn lilv_plugins_begin(plugins: *const Void) -> *mut Void;
    fn lilv_plugins_is_end(plugins: *const Void, i: *mut Void) -> u8;
    fn lilv_plugins_next(plugins: *const Void, i: *mut Void) -> *mut Void;
}

pub struct Plugins {
    pub(crate) plugins: *const Void,
    pub(crate) world: Rc<World>,
}

impl AsRef<*const Void> for Plugins {
    fn as_ref(&self) -> &*const Void {
        &self.plugins
    }
}

impl<'a> Collection<'a> for Plugins
where
    Self: 'a,
{
    type Target = Plugin;

    fn get(&self, i: *mut Void) -> Self::Target {
        Plugin {
            plugin: unsafe { lilv_plugins_get(self.plugins, i) },
            world: self.world.clone(),
        }
    }
}

impl Plugins {
    /// Get a plugin with the given URI.
    /// Returns `None` if no plugin with `uri` is found.
    pub fn get_by_uri<'a>(&'a self, uri: &Node<Uri>) -> Option<Plugin> {
        let ptr = unsafe { lilv_plugins_get_by_uri(self.plugins, uri.node) };

        if ptr.is_null() {
            None
        } else {
            Some(Plugin {
                plugin: ptr,
                world: self.world.clone(),
            })
        }
    }

    pub fn iter<'a>(&'a self) -> Iter<'a, Self> {
        Iter::new(
            self,
            lilv_plugins_begin,
            lilv_plugins_is_end,
            lilv_plugins_next,
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::NodeImpl;
    use crate::WorldImpl;
    use crate::*;
    use std::ffi::CString;

    #[test]
    fn it_works() {
        let w = World::new().unwrap();
        w.load_all();
        let plugins = w.get_all_plugins();
        for plugin in plugins.iter() {
            println!(
                "{}",
                CString::from(plugin.get_uri().value())
                    .into_string()
                    .unwrap()
            );
        }
    }
}
