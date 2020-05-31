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

#[macro_use]
pub mod macros {
    #[allow(unused_macros)]
    macro_rules! log {
    ($fmt:expr $(,$arg:expr)*) => {
            log::log_internal(format!($fmt $(,$arg)*));
    };
    }
}

#[cfg(nightly)]
#[allow(unused)]
pub fn log_internal(message: String) {
    proc_macro::Diagnostic::new(proc_macro::Level::Note, message).emit();
}

#[cfg(not(nightly))]
#[allow(unused)]
pub fn log_internal(message: String) {
    eprintln!("{}", message)
}
