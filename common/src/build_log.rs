/*
Copyright 2025 Google LLC

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

use std::fmt::{Debug, Display, Formatter};

#[macro_export]
macro_rules! log {
    ($($tokens: tt)*) => {
        for line in format!($($tokens)*).lines() {
            println!("cargo::warning={}", line)
        }
    }
}

#[derive(Debug)]
pub struct FatalBuildScriptError {
    pub message: String,
}

impl Display for FatalBuildScriptError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "lockjaw fatal build script error: {}", self.message)
    }
}

impl std::error::Error for FatalBuildScriptError {}

#[macro_export]
macro_rules! build_script_fatal {
    ($($tokens: tt)*) => {
        return Err(crate::build_log::FatalBuildScriptError{message: format!($($tokens)*)}.into())
    }
}
