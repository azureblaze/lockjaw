/*
Copyright 2020 Google LLC

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
#[allow(dead_code)]
pub struct InjectableFromOtherFile {}

#[lockjaw::component_visible]
struct OtherFilePrivate {}

#[lockjaw::injectable]
impl OtherFilePrivate {
    #[inject]
    pub fn new() -> Self {
        Self {}
    }
}

#[lockjaw::injectable]
impl InjectableFromOtherFile {
    #[inject]
    pub fn new(_p: OtherFilePrivate) -> Self {
        Self {}
    }
}

#[lockjaw::component]
#[allow(dead_code)]
pub trait OtherComponent {
    fn other_file(&self) -> crate::other_file::InjectableFromOtherFile;
}
