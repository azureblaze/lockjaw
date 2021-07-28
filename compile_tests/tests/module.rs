/*
Copyright 2021 Google LLC

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a.rs copy of the License at

    https://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

#[test]
fn module_non_impl() {
    let t = trybuild::TestCases::new();
    t.compile_failed_with("tests/module/module_non_impl.rs", vec!["impl expected"])
}

#[test]
fn module_not_path() {
    let t = trybuild::TestCases::new();
    t.compile_failed_with("tests/module/module_not_path.rs", vec!["path expected"])
}

#[test]
fn module_unknown_metadata() {
    let t = trybuild::TestCases::new();
    t.compile_failed_with(
        "tests/module/module_unknown_metadata.rs",
        vec!["unknown key: foo"],
    )
}

#[test]
fn provides_no_return_type() {
    let t = trybuild::TestCases::new();
    t.compile_failed_with(
        "tests/module/provides_no_return_type.rs",
        vec!["return type expected"],
    )
}

#[test]
fn provides_move_self() {
    let t = trybuild::TestCases::new();
    t.compile_failed_with(
        "tests/module/provides_move_self.rs",
        vec!["modules should not consume self"],
    )
}

#[test]
fn provides_param_not_identifier() {
    let t = trybuild::TestCases::new();
    t.compile_failed_with(
        "tests/module/provides_param_not_identifier.rs",
        vec!["identifier expected"],
    )
}

#[test]
fn binds_no_return_type() {
    let t = trybuild::TestCases::new();
    t.compile_failed_with(
        "tests/module/binds_no_return_type.rs",
        vec!["return type expected"],
    )
}

#[test]
fn binds_return_type_not_component_lifetime() {
    let t = trybuild::TestCases::new();
    t.compile_failed_with(
        "tests/module/binds_return_type_not_component_lifetime.rs",
        vec!["#[binds] methods must return ComponentLifetime<T>"],
    )
}

#[test]
fn binds_no_param() {
    let t = trybuild::TestCases::new();
    t.compile_failed_with(
        "tests/module/binds_no_param.rs",
        vec!["binds method must only take the binding type as parameter"],
    )
}

#[test]
fn binds_more_param() {
    let t = trybuild::TestCases::new();
    t.compile_failed_with(
        "tests/module/binds_more_param.rs",
        vec!["binds method must only take the binding type as parameter"],
    )
}

#[test]
fn binds_self() {
    let t = trybuild::TestCases::new();
    t.compile_failed_with(
        "tests/module/binds_self.rs",
        vec!["binds method must only take the binding type as parameter"],
    )
}

#[test]
fn binds_no_identifier() {
    let t = trybuild::TestCases::new();
    t.compile_failed_with(
        "tests/module/binds_no_identifier.rs",
        vec!["identifier expected"],
    )
}

#[test]
fn binds_has_method_body() {
    let t = trybuild::TestCases::new();
    t.compile_failed_with(
        "tests/module/binds_has_method_body.rs",
        vec!["#[binds] methods must have empty body"],
    )
}

#[test]
fn provides_duplicated_bindings() {
    let t = trybuild::TestCases::new();
    t.compile_failed_with(
        "tests/module/provides_duplicated_bindings.rs",
        vec!["#[module] methods can only be annotated by one of #[provides]/#[binds]/#[binds_option_of]"],
    )
}

#[test]
fn binds_elements_into_vec() {
    let t = trybuild::TestCases::new();
    t.compile_failed_with(
        "tests/module/binds_elements_into_vec.rs",
        vec!["#[binds] methods must return ComponentLifetime<T>"],
    )
}

#[test]
fn provides_elements_into_vec_not_vec() {
    let t = trybuild::TestCases::new();
    t.compile_failed_with(
        "tests/module/provides_elements_into_vec_not_vec.rs",
        vec!["#[elements_into_set] must return Vec<T>"],
    )
}

#[test]
fn qualifier_not_declared() {
    let t = trybuild::TestCases::new();
    t.compile_failed_with(
        "tests/module/qualifier_not_declared.rs",
        vec!["qualifier struct is not annotated with the #[lockjaw::qualifier] attribute"],
    )
}

#[test]
fn into_map_string_key_collision() {
    let t = trybuild::TestCases::new();
    t.compile_failed_with(
        "tests/module/into_map_string_key_collision.rs",
        vec!["found duplicated key String(\"1\")"],
    )
}

#[test]
fn into_map_i32_key_collision() {
    let t = trybuild::TestCases::new();
    t.compile_failed_with(
        "tests/module/into_map_i32_key_collision.rs",
        vec!["found duplicated key I32(1)"],
    )
}

#[test]
fn module_not_installed_in_defined_component() {
    let t = trybuild::TestCases::new();
    t.compile_failed_with(
        "tests/module/module_not_installed_in_defined_component.rs",
        vec![
            "#[module]",
            "but the component is not annotated with #[define_component] or #[define_subcomponent]",
        ],
    )
}
