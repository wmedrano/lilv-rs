use std::env;

fn main() {
    if env::var("DOCS_RS") == Ok("1".to_string()) {
        return;
    }
    pkg_config::Config::new()
        .atleast_version("0.24.2")
        .probe("lilv-0")
        .expect("lilv-0 could not be found with pkg_config.");
}
