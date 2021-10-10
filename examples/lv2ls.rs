fn main() {
    let world = lilv::World::new();
    world.load_all();

    let show_names = false;

    let print = |plugin: lilv::Plugin| {
        if show_names {
            String::from(plugin.name().as_str().unwrap())
        } else {
            String::from(plugin.uri().as_uri().unwrap())
        }
    };

    let plugins = world
        .plugins()
        .filter(lilv::Plugin::verify)
        .map(print)
        .collect::<Vec<_>>();

    debug_assert_eq!(world.plugins_count(), plugins.len());

    for uri in plugins {
        println!("{}", uri);
    }
}
