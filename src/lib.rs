pub mod instance;
pub mod node;
pub mod plugin;
pub mod plugin_class;
pub mod port;
pub mod ui;
pub mod world;

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RATE: f64 = 44100.0;

    #[test]
    fn test_all_plugins_uri() {
        let world = world::World::with_load_all();
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

        let world = world::World::with_load_all();
        let tests = [TestCase {
            uri: "http://lv2plug.in/plugins/eg-amp",
            num_ports: 3,
        }];

        for test_case in tests {
            let plugin = world
                .plugins()
                .plugin(&world.new_uri(test_case.uri))
                .unwrap_or_else(|| {
                    panic!("{}: Plugin not found.", test_case.uri);
                });
            assert_eq!(
                test_case.num_ports,
                plugin.ports_count(),
                "{}: Wrong ports",
                test_case.uri
            );
            let required_features = plugin.required_features();
            assert_eq!(
                0,
                required_features.count(),
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
