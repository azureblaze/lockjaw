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

use backtrace::Backtrace;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use quote::quote_spanned;
use std::panic::UnwindSafe;

struct Panic {
    backtrace: Backtrace,
    msg: String,
}

static mut PANIC: Option<Panic> = None;

pub fn handle_error<F>(f: F) -> proc_macro::TokenStream
where
    F: FnOnce() -> Result<TokenStream, TokenStream> + UnwindSafe,
{
    unsafe {
        PANIC = None;

        std::panic::set_hook(Box::new(|info| {
            PANIC = Some(Panic {
                backtrace: Backtrace::new(),
                msg: info.to_string(),
            });
        }));

        let result = std::panic::catch_unwind(|| f());
        let _ = std::panic::take_hook();

        if result.is_ok() {
            return match result.unwrap() {
                Ok(r) => r.into(),
                Err(r) => r.into(),
            };
        }
        if let Some(ref p) = PANIC {
            let msg = format!("lockjaw panicked:\n{}\n{:#?}", p.msg, p.backtrace);
            return quote! {
                compile_error!(#msg);
            }
            .into();
        } else {
            std::panic::resume_unwind(result.err().unwrap())
        }
    }
}

#[must_use]
pub fn compile_error<T>(message: &str) -> Result<T, TokenStream> {
    Err(quote! {
        compile_error!(#message);
    })
}

#[must_use]
pub fn spanned_compile_error<T>(span: Span, message: &str) -> Result<T, TokenStream> {
    Err(quote_spanned! {span=>
        compile_error!(#message);
    })
}

pub trait CompileError<T> {
    fn map_compile_error(self, message: &str) -> Result<T, TokenStream>;

    fn map_spanned_compile_error(self, span: Span, message: &str) -> Result<T, TokenStream>;
}

impl<T> CompileError<T> for Option<T> {
    fn map_compile_error(self, message: &str) -> Result<T, TokenStream> {
        self.ok_or(quote! {
            compile_error!(#message);
        })
    }
    fn map_spanned_compile_error(self, span: Span, message: &str) -> Result<T, TokenStream> {
        self.ok_or(quote_spanned! {span=>
            compile_error!(#message);
        })
    }
}

impl<T, E> CompileError<T> for Result<T, E> {
    fn map_compile_error(self, message: &str) -> Result<T, TokenStream> {
        self.map_err(|_| {
            quote! {
                compile_error!(#message);
            }
        })
    }

    fn map_spanned_compile_error(self, span: Span, message: &str) -> Result<T, TokenStream> {
        self.map_err(|_| {
            quote_spanned! {span=>
                compile_error!(#message);
            }
        })
    }
}
