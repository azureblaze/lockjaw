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
fn component() {
    {
        let t = trybuild::TestCases::new();
        t.compile_failed_with(
            set_src_path("tests/component/scope_not_declared.rs"),
            vec!["cannot find trait `bar` in this scope"],
        )
    }
    {
        let t = trybuild::TestCases::new();
        t.compile_failed_with(
            set_src_path("tests/component/component_non_trait.rs"),
            vec!["trait expected"],
        )
    }
    {
        let t = trybuild::TestCases::new();
        t.compile_failed_with(
            set_src_path("tests/component/component_provision_no_return_type.rs"),
            vec!["return type expected for component provisions"],
        )
    }
    {
        let t = trybuild::TestCases::new();
        t.compile_failed_with(
            set_src_path("tests/component/component_param_not_meta_name_value.rs"),
            vec!["unknown key: asdf"],
        )
    }
    {
        let t = trybuild::TestCases::new();
        t.compile_failed_with(
            set_src_path("tests/component/component_key_not_identifier.rs"),
            vec!["FieldValue (key: value, ...) expected"],
        )
    }
    {
        let t = trybuild::TestCases::new();
        t.compile_failed_with(
            set_src_path("tests/component/component_modules_not_path.rs"),
            vec!["path expected for modules"],
        )
    }
    {
        let t = trybuild::TestCases::new();
        t.compile_failed_with(
            set_src_path("tests/component/component_trait_object_provision_no_lifetime.rs"),
            vec!["trait object return type may depend on scoped objects, and must have lifetime bounded by the component by wrapping with lockjaw::Cl<>"],
        )
    }
    {
        let t = trybuild::TestCases::new();
        t.compile_failed_with(
            set_src_path("tests/component/component_provision_non_scoped_of_scoped.rs"),
            vec!["unable to provide scoped binding as regular type"],
        )
    }
    {
        let t = trybuild::TestCases::new();
        t.compile_failed_with(
            set_src_path("tests/component/component_unknown_metadata.rs"),
            vec!["unknown key: foo"],
        )
    }
    {
        let t = trybuild::TestCases::new();
        t.compile_failed_with(
            set_src_path("tests/component/entry_point_not_installed_in_defined_component.rs"),
            vec![
                "#[entry_point]",
                "the component is not annotated with #[define_component] or #[define_subcomponent]",
            ],
        )
    }
}
