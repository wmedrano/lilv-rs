use crate::collection::Collection;
use crate::collection::Iter;
use crate::node::Node;
use crate::plugin::Plugin;
use crate::world::World;
use crate::Void;
use lilv_sys::*;
use std::rc::Rc;

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

    unsafe fn get(&self, i: *mut Void) -> Self::Target {
        Plugin {
            plugin: lilv_plugins_get(self.plugins, i),
            world: self.world.clone(),
        }
    }
}

impl Plugins {
    /// Get a plugin with the given URI.
    /// Returns `None` if no plugin with `uri` is found.
    pub fn get_by_uri<'a>(&'a self, uri: &Node) -> Option<Plugin> {
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

    pub fn iter(&self) -> Iter<'_, Self> {
        Iter::new(
            self,
            lilv_plugins_begin,
            lilv_plugins_is_end,
            lilv_plugins_next,
        )
    }

    pub fn size(&self) -> usize {
        unsafe { lilv_plugins_size(self.plugins) as usize }
    }
}

#[cfg(test)]
mod tests {
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
                CString::from(plugin.get_uri().value().into_string())
                    .into_string()
                    .unwrap()
            );
        }
    }
}
