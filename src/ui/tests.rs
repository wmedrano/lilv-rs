use super::*;
use crate::World;
use std::fs;

const PLUGIN: &str = "urn:lilv:test-plugin";
const UI_URI: &str = "urn:lilv:test-ui";
const CLASS: &str = "http://lv2plug.in/ns/extensions/ui#X11UI";

fn fixture(with_ui: bool) -> (tempfile::TempDir, World, Plugin) {
    let directory = tempfile::tempdir().unwrap();
    let bundle = directory.path().join("test.lv2");
    fs::create_dir(&bundle).unwrap();
    let ui_metadata = if with_ui {
        r#"
<urn:lilv:test-plugin> ui:ui <urn:lilv:test-ui> .
<urn:lilv:test-ui> a ui:X11UI ; ui:binary <test-ui.so> .
"#
    } else {
        ""
    };
    fs::write(
        bundle.join("manifest.ttl"),
        format!(
            r#"
@prefix lv2: <http://lv2plug.in/ns/lv2core#> .
@prefix ui: <http://lv2plug.in/ns/extensions/ui#> .
<urn:lilv:test-plugin> a lv2:Plugin ; lv2:binary <test.so> .
{}
"#,
            ui_metadata
        ),
    )
    .unwrap();
    let world = World::new();
    let bundle_uri = world.new_file_uri(None, &format!("{}/", bundle.display()));
    world.load_bundle(&bundle_uri);
    let plugin = world.plugins().plugin(&world.new_uri(PLUGIN)).unwrap();
    (directory, world, plugin)
}

#[test]
fn lookup_and_iteration() {
    let (_directory, world, plugin) = fixture(true);
    let uis = plugin.uis().unwrap();
    assert_eq!(uis.count(), 1);
    assert!(uis.get_by_uri(&world.new_uri("urn:lilv:missing")).is_none());
    let ui = uis.get_by_uri(&world.new_uri(UI_URI)).unwrap();
    assert_eq!(ui.uri().as_uri(), Some(UI_URI));
    assert!(ui.is_a(&world.new_uri(CLASS)));
    let mut iter = uis.iter();
    assert_eq!(iter.next().unwrap().uri().as_uri(), Some(UI_URI));
    assert!(iter.next().is_none());
    assert!(iter.next().is_none());
}

#[test]
fn no_uis() {
    let (_directory, _world, plugin) = fixture(false);
    // Native Lilv may represent an empty collection with a null pointer.
    if let Some(uis) = plugin.uis() {
        assert_eq!(uis.count(), 0);
        let mut iter = uis.into_iter();
        assert!(iter.next().is_none());
        assert!(iter.next().is_none());
    }
}

#[test]
fn collection_is_shared_until_the_last_dependent_drops() {
    let (_directory, world, plugin) = fixture(true);
    let uis = plugin.uis().unwrap();
    let owner = Rc::downgrade(&uis.owner);
    let clone = uis.clone();
    let mut iter = uis.into_iter();
    drop(clone);
    assert!(owner.upgrade().is_some());
    let ui = iter.next().unwrap();
    drop(iter);
    drop(plugin);
    drop(world);
    assert_eq!(ui.uri().as_uri(), Some(UI_URI));
    let classes = ui.classes();
    drop(ui);
    assert!(owner.upgrade().is_some());
    assert_eq!(classes.count(), 1);
    let mut classes = classes.into_iter();
    let class = classes.next().unwrap();
    assert!(classes.next().is_none());
    assert!(classes.next().is_none());
    drop(classes);
    assert!(owner.upgrade().is_none());
    // A yielded class node owns a copy independent of both collections.
    assert_eq!(class.as_uri(), Some(CLASS));
}

struct SupportX11;
impl UISupport for SupportX11 {
    fn supported(container: &str, ui: &str) -> UISupportQuality {
        UISupportQuality(u32::from(container == CLASS && ui == CLASS))
    }
}

#[test]
fn returned_metadata_outlives_ui_collection_and_world() {
    let (directory, world, plugin) = fixture(true);
    let uis = plugin.uis().unwrap();
    let owner = Rc::downgrade(&uis.owner);
    let ui = uis.get_by_uri(&world.new_uri(UI_URI)).unwrap();
    let uri = ui.uri();
    let bundle = ui.bundle_uri().unwrap();
    let binary = ui.binary_uri().unwrap();
    let class = ui.classes().into_iter().next().unwrap();
    let mut supported_type = Some(world.new_uri("urn:lilv:previous-type"));
    assert_eq!(
        ui.is_supported::<SupportX11>(&world.new_uri(CLASS), Some(&mut supported_type)),
        UISupportQuality(1)
    );
    let supported_type = supported_type.unwrap();
    drop(ui);
    drop(uis);
    drop(plugin);
    drop(world);
    assert!(owner.upgrade().is_none());
    assert_eq!(uri.as_uri(), Some(UI_URI));
    assert_eq!(class.as_uri(), Some(CLASS));
    assert_eq!(supported_type.as_uri(), Some(CLASS));
    assert_eq!(
        bundle.path().unwrap().1,
        format!("{}/test.lv2/", directory.path().display())
    );
    assert_eq!(
        binary.path().unwrap().1,
        format!("{}/test.lv2/test-ui.so", directory.path().display())
    );
}
