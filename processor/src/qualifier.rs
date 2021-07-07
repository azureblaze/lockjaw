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

use crate::error::CompileError;
use crate::manifest::with_manifest;
use crate::prologue::prologue_check;
use crate::type_data::TypeData;
use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;

pub fn handle_qualifier_attribute(
    _attr: TokenStream,
    input: TokenStream,
) -> Result<TokenStream, TokenStream> {
    let span = input.span();
    let item: syn::ItemStruct =
        syn::parse2(input).map_spanned_compile_error(span, "struct block expected")?;

    let r: Result<(), TokenStream> = with_manifest(|mut manifest| {
        Ok(manifest.qualifiers.push(TypeData::from_local(
            &item.ident.to_string(),
            item.ident.span(),
        )?))
    });
    r?;

    let prologue_check = prologue_check(item.span());
    Ok(quote! {
        #item
        #prologue_check
    })
}
