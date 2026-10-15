//! The server's SQL names only the `template` schema. Core state comes through
//! veoveo-sdk-rust and core's runtime API.

#[test]
fn sql_names_only_the_template_schema() {
    veoveo_sdk_rust::testing::assert_sql_confined(env!("CARGO_MANIFEST_DIR"), "template");
}
