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

use crate::manifest_parser::Mod;
use proc_macro2::Span;
use std::fmt::{Debug, Display, Formatter};

#[doc(hidden)]
#[macro_export]
macro_rules! log {
    ($($tokens: tt)*) => {
        for line in format!($($tokens)*).lines() {
            println!("cargo::warning={}", line)
        }
    }
}

#[derive(Debug)]
pub(crate) struct FatalBuildScriptError {
    pub span: SpanData,
    pub message: String,
}

#[derive(Debug)]
pub(crate) struct SpanData {
    location: String,
    line: String,
    marker: String,
}

impl SpanData {
    pub fn from_span(span: Span, mod_: &Mod) -> Self {
        SpanData {
            location: format!(
                "{}:{}:{}",
                mod_.source_file,
                span.start().line,
                span.start().column
            ),
            line: mod_
                .source
                .lines()
                .nth(span.start().line - 1)
                .unwrap_or("(invalid)")
                .to_string(),
            marker: format!(
                "{}{}",
                " ".repeat(span.start().column),
                "^".repeat(span.end().column - span.start().column)
            ),
        }
    }
}

impl Display for FatalBuildScriptError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "lockjaw fatal build script error:in {}\n{}\n{}\n{}",
            self.span.location, self.span.line, self.span.marker, self.message
        )
    }
}

impl std::error::Error for FatalBuildScriptError {}

#[doc(hidden)]
#[macro_export]
macro_rules! build_script_fatal {
    ($span:expr, $mod_:expr, $($tokens: tt)*) => {
        return Err(crate::build_log::FatalBuildScriptError{span: crate::build_log::SpanData::from_span($span, $mod_), message: format!($($tokens)*)}.into())
    }
}
