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
use compile_tests::set_src_path;

#[test]
fn module() {
    {
        let t = trybuild::TestCases::new();
        t.compile_failed_with(
            set_src_path("tests/module/module_non_impl.rs"),
            vec!["impl expected"],
        )
    }
    {
        let t = trybuild::TestCases::new();
        t.compile_failed_with(
            set_src_path("tests/module/module_not_path.rs"),
            vec!["path expected"],
        )
    }
    {
        let t = trybuild::TestCases::new();
        t.compile_failed_with(
            set_src_path("tests/module/module_unknown_metadata.rs"),
            vec!["unknown key: foo"],
        )
    }
    {
        let t = trybuild::TestCases::new();
        t.compile_failed_with(
            set_src_path("tests/module/provides_no_return_type.rs"),
            vec!["return type expected"],
        )
    }
    {
        let t = trybuild::TestCases::new();
        t.compile_failed_with(
            set_src_path("tests/module/provides_move_self.rs"),
            vec!["modules should not consume self"],
        )
    }
    {
        let t = trybuild::TestCases::new();
        t.compile_failed_with(
            set_src_path("tests/module/provides_param_not_identifier.rs"),
            vec!["identifier expected"],
        )
    }
    {
        let t = trybuild::TestCases::new();
        t.compile_failed_with(
            set_src_path("tests/module/binds_no_return_type.rs"),
            vec!["return type expected"],
        )
    }
    {
        let t = trybuild::TestCases::new();
        t.compile_failed_with(
            set_src_path("tests/module/binds_return_type_not_component_lifetime.rs"),
            vec!["#[binds] methods must return Cl<T>"],
        )
    }
    {
        let t = trybuild::TestCases::new();
        t.compile_failed_with(
            set_src_path("tests/module/binds_no_param.rs"),
            vec!["binds method must only take the binding type as parameter"],
        )
    }
    {
        let t = trybuild::TestCases::new();
        t.compile_failed_with(
            set_src_path("tests/module/binds_more_param.rs"),
            vec!["binds method must only take the binding type as parameter"],
        )
    }
    {
        let t = trybuild::TestCases::new();
        t.compile_failed_with(
            set_src_path("tests/module/binds_self.rs"),
            vec!["binds method must only take the binding type as parameter"],
        )
    }
    {
        let t = trybuild::TestCases::new();
        t.compile_failed_with(
            set_src_path("tests/module/binds_no_identifier.rs"),
            vec!["identifier expected"],
        )
    }
    {
        let t = trybuild::TestCases::new();
        t.compile_failed_with(
            set_src_path("tests/module/binds_has_method_body.rs"),
            vec!["#[binds] methods must have empty body"],
        )
    }
    {
        let t = trybuild::TestCases::new();
        t.compile_failed_with(set_src_path(
            "tests/module/provides_duplicated_bindings.rs"),
                              vec!["#[module] methods can only be annotated by one of #[provides]/#[binds]/#[binds_option_of]"],
        )
    }
    {
        let t = trybuild::TestCases::new();
        t.compile_failed_with(
            set_src_path("tests/module/binds_elements_into_vec.rs"),
            vec!["#[binds] methods must return Cl<T>"],
        )
    }
    {
        let t = trybuild::TestCases::new();
        t.compile_failed_with(
            set_src_path("tests/module/provides_elements_into_vec_not_vec.rs"),
            vec!["#[elements_into_set] must return Vec<T>"],
        )
    }
    {
        let t = trybuild::TestCases::new();
        t.compile_failed_with(
            set_src_path("tests/module/qualifier_not_declared.rs"),
            vec!["qualifier struct is not annotated with the #[lockjaw::qualifier] attribute"],
        )
    }
    {
        let t = trybuild::TestCases::new();
        t.compile_failed_with(
            set_src_path("tests/module/into_map_string_key_collision.rs"),
            vec!["found duplicated key String(\"1\")"],
        )
    }
    {
        let t = trybuild::TestCases::new();
        t.compile_failed_with(
            set_src_path("tests/module/into_map_i32_key_collision.rs"),
            vec!["found duplicated key I32(1)"],
        )
    }
    {
        let t = trybuild::TestCases::new();
        t.compile_failed_with(
            set_src_path("tests/module/module_not_installed_in_defined_component.rs"),
            vec![
                "#[module]",
                "but the component is not annotated with #[define_component] or #[define_subcomponent]",
            ],
        )
    }
    {
        let t = trybuild::TestCases::new();
        t.compile_failed_with(
            set_src_path("tests/module/module_multibinds_not_vec_or_hashmap.rs"),
            vec!["#[multibinds] methods must return Vec<T> or HashMap<K,V>"],
        )
    }
    {
        let t = trybuild::TestCases::new();
        t.compile_failed_with(
            set_src_path("tests/module/module_multibinds_has_body.rs"),
            vec!["#[multibinds] methods must have empty body"],
        )
    }
    {
        let t = trybuild::TestCases::new();
        t.compile_failed_with(
            set_src_path("tests/module/module_multibinds_has_args.rs"),
            vec!["#[multibinds] method must take no arguments"],
        )
    }
    {
        let t = trybuild::TestCases::new();
        t.compile_failed_with(set_src_path(
            "tests/module/module_multibinds_duplicated_binds.rs"),
                              vec![
                                  "#[module] methods can only be annotated by one of #[provides]/#[binds]/#[binds_option_of]/#[multibinds]",
                              ],
        )
    }
}
