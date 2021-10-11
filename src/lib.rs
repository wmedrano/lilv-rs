mod instance;
mod node;
mod plugin;
mod plugin_class;
mod port;
mod state;
mod ui;
mod uis;
mod world;

pub use instance::*;
pub use node::*;
pub use plugin::*;
pub use plugin_class::*;
pub use port::*;
pub use state::*;
pub use ui::*;
pub use uis::*;
pub use world::*;

pub(crate) fn make_c_string(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    if bytes[bytes.len() - 1] == 0 {
        None
    } else {
        Some(format!("{}\0", value))
    }
}

pub(crate) fn choose_string<'a>(a: &'a str, b: &'a Option<String>) -> *const i8 {
    b.as_ref().map_or(a, String::as_str).as_ptr().cast::<i8>()
}

#[cfg(test)]
mod tests {
    use super::*;

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
