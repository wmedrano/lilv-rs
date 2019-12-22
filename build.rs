fn main() {
    pkg_config::Config::new()
        .atleast_version("0.24.2")
        .probe("lilv-0")
        .unwrap();
}
