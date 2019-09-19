# lilv-rs

This is a Rust wrapper for [Lilv](http://drobilla.net/software/lilv),
the LV2 host library.

**Please be cautious when using this crate!** It may work or break;
for the moment its intended use is as a dependency for a certain project.
It attempts to wrap everything nicely in idiomatic ways, but all functionality
is not tested.

## Completeness

This crate targets the latest version of Lilv, which is at the time of
writing, 0.24.2.

Below is a list of functions that are wrapped, and those that aren't.

### Done

    lilv_file_uri_parse
    lilv_world_free
    lilv_world_new
    lilv_world_set_option
    lilv_world_load_all
    lilv_world_load_bundle
    lilv_world_load_specifications
    lilv_world_load_plugin_classes
    lilv_world_unload_bundle
    lilv_world_load_resource
    lilv_world_unload_resource
    lilv_world_get_all_plugins
    lilv_world_find_nodes
    lilv_world_get
    lilv_world_ask
    lilv_world_get_symbol
    lilv_world_get_plugin_class
    lilv_world_get_plugin_classes
    lilv_new_uri
    lilv_new_file_uri
    lilv_new_string
    lilv_new_int
    lilv_new_float
    lilv_new_bool
    lilv_port_is_a
    lilv_plugins_size
    lilv_plugins_begin
    lilv_plugins_get
    lilv_plugins_next
    lilv_plugins_is_end
    lilv_plugins_get_by_uri
    lilv_plugin_get_uri
    lilv_plugin_get_num_ports
    lilv_plugin_get_port_ranges_float
    lilv_plugin_get_port_by_index
    lilv_plugin_instantiate
    lilv_plugin_get_bundle_uri
    lilv_plugin_get_data_uris
    lilv_plugin_get_library_uri
    lilv_plugin_get_name
    lilv_plugin_get_value
    lilv_plugin_has_feature
    lilv_plugin_get_supported_features
    lilv_plugin_get_required_features
    lilv_plugin_get_optional_features
    lilv_plugin_has_extension_data
    lilv_plugin_get_extension_data
    lilv_plugin_has_latency
    lilv_plugin_get_latency_port_index
    lilv_plugin_get_port_by_symbol
    lilv_plugin_get_port_by_designation
    lilv_plugin_get_project
    lilv_plugin_get_author_name
    lilv_plugin_get_author_email
    lilv_plugin_get_author_homepage
    lilv_plugin_is_replaced
    lilv_plugin_get_related
    lilv_plugin_verify
    lilv_plugin_get_class
    lilv_plugin_get_num_ports_of_class [reimplemented]
    lilv_plugin_get_uis
    lilv_node_duplicate
    lilv_node_equals
    lilv_node_is_uri
    lilv_node_as_uri
    lilv_node_is_blank
    lilv_node_as_blank
    lilv_node_is_literal
    lilv_node_is_string
    lilv_node_as_string
    lilv_node_is_float
    lilv_node_as_float
    lilv_node_is_int
    lilv_node_as_int
    lilv_node_is_bool
    lilv_node_as_bool
    lilv_node_get_path
    lilv_node_free
    lilv_instance_connect_port
    lilv_instance_activate
    lilv_instance_run
    lilv_instance_deactivate
    lilv_instance_free
    lilv_instance_get_uri
    lilv_instance_get_extension_data
    lilv_instance_get_descriptor
    lilv_instance_get_handle
    lilv_nodes_free
    lilv_nodes_size
    lilv_nodes_begin
    lilv_nodes_get
    lilv_nodes_next
    lilv_nodes_is_end
    lilv_nodes_contains
    lilv_nodes_merge
    lilv_port_get_node
    lilv_port_get_value
    lilv_port_get
    lilv_port_get_properties
    lilv_port_has_property
    lilv_port_supports_event
    lilv_port_get_index
    lilv_port_get_symbol
    lilv_port_get_name
    lilv_port_get_classes
    lilv_port_get_range
    lilv_port_get_scale_points
    lilv_free
    lilv_node_get_turtle_token
    lilv_plugin_class_get_parent_uri
    lilv_plugin_class_get_uri
    lilv_plugin_class_get_label
    lilv_plugin_class_get_children
    lilv_plugin_classes_free
    lilv_plugin_classes_size
    lilv_plugin_classes_begin
    lilv_plugin_classes_get
    lilv_plugin_classes_next
    lilv_plugin_classes_is_end
    lilv_plugin_classes_get_by_uri
    lilv_scale_points_free
    lilv_scale_points_size
    lilv_scale_points_begin
    lilv_scale_points_get
    lilv_scale_points_next
    lilv_scale_points_is_end
    lilv_ui_get_uri
    lilv_ui_get_classes
    lilv_ui_is_a
    lilv_ui_is_supported
    lilv_ui_get_bundle_uri
    lilv_ui_get_binary_uri
    lilv_uis_free
    lilv_uis_size
    lilv_uis_begin
    lilv_uis_get
    lilv_uis_next
    lilv_uis_is_end
    lilv_uis_get_by_uri
    lilv_scale_point_get_label
    lilv_scale_point_get_value
    lilv_state_new_from_world
    lilv_state_new_from_file
    lilv_state_new_from_string
    lilv_state_new_from_instance
    lilv_state_free
    lilv_state_equals
    lilv_state_get_num_properties
    lilv_state_get_plugin_uri
    lilv_state_get_uri
    lilv_state_get_label
    lilv_state_set_label
    lilv_state_set_metadata
    lilv_state_emit_port_values
    lilv_state_restore
    lilv_state_save
    lilv_state_to_string
    lilv_state_delete

### Missing

    lilv_uri_to_path [deprecated]
    lilv_nodes_get_first [unnecessary]
    lilv_plugin_write_description [too much C]
    lilv_plugin_write_manifest_entry [too much C]
