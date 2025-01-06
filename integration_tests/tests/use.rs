/*
Copyright 2021 Google LLC

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    https://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

use lockjaw::{epilogue, prologue};

mod other_file;

prologue!("tests/use.rs");

mod foo {
    use lockjaw::injectable as use_as_injectable;

    pub struct Inner;

    #[use_as_injectable]
    impl Inner {
        #[inject]
        pub fn new() -> Self {
            Inner {}
        }
    }

    pub struct UseAs;

    #[use_as_injectable]
    impl UseAs {
        #[inject]
        pub fn new() -> Self {
            UseAs {}
        }
    }

    pub mod bar {
        use lockjaw::injectable;

        pub struct Inner2;

        #[injectable]
        impl Inner2 {
            #[inject]
            pub fn new() -> Self {
                Inner2 {}
            }
        }
    }
}

use foo::bar;
use foo::Inner;
use foo::UseAs as use_as;
use test_dep::DepInjectable;

#[lockjaw::define_component]
pub trait MyComponent {
    fn inner(&self) -> Inner;
    fn dep_injectable(&self) -> DepInjectable;
    fn use_as(&self) -> use_as;
    fn indirect_use(&self) -> bar::Inner2;
    fn other_file(&self) -> other_file::InjectableFromOtherFile;
}

#[test]
pub fn main() {
    lockjaw_init();
    let _component: Box<dyn MyComponent> = <dyn MyComponent>::new();
}
epilogue!();
