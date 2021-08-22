mod collection;
mod instance;
mod node;
mod nodes;
mod plugin;
mod plugin_class;
mod plugin_classes;
mod plugins;
mod port;
mod scale_point;
mod scale_points;
mod state;
mod ui;
mod uis;
mod world;

pub use collection::*;
pub use instance::*;
pub use node::*;
pub use nodes::*;
pub use plugin::*;
pub use plugin_class::*;
pub use plugin_classes::*;
pub use plugins::*;
pub use port::*;
pub use scale_point::*;
pub use scale_points::*;
pub use state::*;
pub use ui::*;
pub use uis::*;
pub use world::*;

pub(crate) fn make_c_string(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    if bytes[bytes.len() - 1] != 0 {
        Some(format!("{}\0", value))
    } else {
        None
    }
}

pub(crate) fn choose_string<'a>(a: &'a str, b: &'a Option<String>) -> *const i8 {
    b.as_ref().map(String::as_str).unwrap_or(a).as_ptr() as _
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn it_works() {
        let world = World::new();
        world.load_all();

        for plugin in world.plugins() {
            println!("{:?}", plugin.uri().as_str());
        }

        let node = world.new_uri("http://lv2plug.in/plugins/eg-amp");
        let plugin = world.plugin(&node).unwrap();
        assert_eq!(3, plugin.num_ports());
        println!(
            "plugin <{}> has {} ports",
            node.as_uri().unwrap(),
            plugin.num_ports(),
        );
    }
}
