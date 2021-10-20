mod instance;
mod node;
mod plugin;
mod plugin_class;
mod port;
mod ui;
mod uis;
mod world;

pub use instance::*;
pub use node::*;
pub use plugin::*;
pub use plugin_class::*;
pub use port::*;
pub use ui::*;
pub use uis::*;
pub use world::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let world = World::with_load_all();

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
