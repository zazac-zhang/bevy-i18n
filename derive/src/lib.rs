//! Derive macro for bevy_i18n Localizable trait.
//!
//! # Usage
//!
//! ```ignore
//! use bevy::prelude::*;
//! use bevy_i18n::prelude::*;
//!
//! #[derive(I18n, Component)]
//! struct DialogBox {
//!     #[i18n(key = "dialog.title")]
//!     title: String,
//!     #[i18n(key = "dialog.body")]
//!     content: String,
//!     color: Color, // non-String field, auto-ignored
//! }
//!
//! // With namespace
//! #[derive(I18n, Component)]
//! #[i18n(namespace = "hud")]
//! struct HUD {
//!     #[i18n(key = "score")]   // → "hud.score"
//!     score_text: String,
//! }
//! ```

mod attr;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, DataStruct, Fields};
use attr::{parse_struct_attrs, parse_field_attrs};

#[proc_macro_derive(I18n, attributes(i18n))]
pub fn derive_i18n(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let struct_config = match parse_struct_attrs(&input.attrs) {
        Ok(c) => c,
        Err(e) => return e.to_compile_error().into(),
    };

    let fields = match &input.data {
        Data::Struct(DataStruct { fields: Fields::Named(f), .. }) => &f.named,
        _ => {
            return quote! {
                compile_error!("#[derive(I18n)] only supports structs with named fields");
            }
            .into();
        }
    };

    let mut translatable_fields: Vec<(syn::Ident, String)> = Vec::new();

    for field in fields {
        let field_ident = field.ident.as_ref().unwrap();

        // Skip non-String fields
        let is_string = if let syn::Type::Path(type_path) = &field.ty {
            type_path
                .path
                .segments
                .last()
                .map(|s| s.ident == "String")
                .unwrap_or(false)
        } else {
            false
        };

        let field_attr = match parse_field_attrs(&field.attrs) {
            Ok(a) => a,
            Err(e) => return e.to_compile_error().into(),
        };

        match field_attr {
            Some(attr) if attr.skip => {
                continue;
            }
            Some(attr) => {
                let key = attr.key.unwrap_or_else(|| field_ident.to_string());
                let full_key = if let Some(ns) = &struct_config.namespace {
                    format!("{}.{}", ns, key)
                } else {
                    key
                };
                translatable_fields.push((field_ident.clone(), full_key));
            }
            None => {
                if is_string {
                    let key = field_ident.to_string();
                    let full_key = if let Some(ns) = &struct_config.namespace {
                        format!("{}.{}", ns, key)
                    } else {
                        key
                    };
                    translatable_fields.push((field_ident.clone(), full_key));
                }
            }
        }
    }

    if translatable_fields.is_empty() {
        return quote! {
            compile_error!("#[derive(I18n)] requires at least one String field (use #[i18n(skip)] to silence)");
        }
        .into();
    }

    let field_names: Vec<&syn::Ident> = translatable_fields.iter().map(|(f, _)| f).collect();
    let keys: Vec<&String> = translatable_fields.iter().map(|(_, k)| k).collect();

    let expanded = quote! {
        impl ::bevy_i18n::Localizable for #name {
            fn translations() -> &'static [(&'static str, &'static str)] {
                &[
                    #((stringify!(#field_names), #keys)),*
                ]
            }

            fn set_field(&mut self, field_name: &str, value: &str) {
                match field_name {
                    #(stringify!(#field_names) => self.#field_names = value.to_string()),*,
                    _ => {}
                }
            }
        }
    };

    TokenStream::from(expanded)
}
