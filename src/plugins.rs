use crate::node::Node;
use crate::plugin::Plugin;
use crate::world::World;
use lilv_sys::*;
use std::rc::Rc;

pub struct Plugins {
    world: Rc<World>,
    plugins: *const LilvPlugins,
}

impl Plugins {
    /// Get a plugin with the given URI.
    /// Returns `None` if no plugin with `uri` is found.
    pub fn by_uri<'a>(&'a self, uri: &Node) -> Option<Plugin> {
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

    pub fn len(&self) -> usize {
        unsafe { lilv_plugins_size(self.plugins) as usize }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Iterator over all the plugins.
    pub fn iter(&self) -> PluginsIter<'_> {
        PluginsIter {
            plugins: self,
            iter: unsafe { lilv_plugins_begin(self.plugins) },
        }
    }
}

pub struct PluginsIter<'a> {
    plugins: &'a Plugins,
    iter: *mut LilvIter,
}

impl<'a> Iterator for PluginsIter<'a> {
    type Item = Plugin;

    fn next(&mut self) -> Option<Plugin> {
        let ptr = unsafe { lilv_plugins_get(self.plugins.plugins, self.iter) };
        if ptr.is_null() {
            None
        } else {
            self.iter = unsafe { lilv_plugins_next(self.plugins.plugins, self.iter) };
            Some(Plugin {
                plugin: ptr,
                world: self.plugins.world.clone(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::WorldImpl;
    use crate::*;
    use std::ffi::CString;

    #[test]
    fn it_works() {
        let w = World::with_load_all().unwrap();
        let plugins = w.all_plugins();
        for plugin in plugins.iter() {
            println!(
                "{}",
                CString::from(plugin.uri().value().into_string())
                    .into_string()
                    .unwrap()
            );
        }
    }
}
