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

    const SAMPLE_RATE: f64 = 44100.0;

    #[test]
    fn test_all_plugins_uri() {
        let world = World::with_load_all();
        for plugin in world.plugins() {
            assert!(plugin.uri().as_uri().is_some());
        }
    }

    #[test]
    fn test_plugins_end_to_end() {
        struct TestCase {
            uri: &'static str,
            num_ports: usize,
        }

        let world = World::with_load_all();
        let tests = [TestCase {
            uri: "http://lv2plug.in/plugins/eg-amp",
            num_ports: 3,
        }];

        for test_case in tests {
            let plugin = world
                .plugin(&world.new_uri(test_case.uri))
                .unwrap_or_else(|| {
                    panic!("{}: Plugin not found.", test_case.uri);
                });
            assert_eq!(
                test_case.num_ports,
                plugin.num_ports(),
                "{}: Wrong ports",
                test_case.uri
            );
            let required_features = plugin.required_features();
            assert_eq!(
                0,
                required_features.size(),
                "{}: Required features not supported but found {:?}",
                test_case.uri,
                required_features
            );
            let instance = unsafe { plugin.instantiate(SAMPLE_RATE, &[]) }.unwrap_or_else(|| {
                panic!("{}: Plugin instantiation failed", test_case.uri);
            });
            assert!(
                unsafe { instance.activate() }.is_some(),
                "{}: Failed to activate",
                test_case.uri
            );
        }
    }
}
