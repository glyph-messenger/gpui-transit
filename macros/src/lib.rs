//! Procedural macros for gpui-transit navigation framework.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Automatically implements `RouteConfig` for a struct or enum.
///
/// # Attributes
/// - `#[route(name = "...")]`: Optional custom route name string.
/// - `#[route(result = "...")]`: Implements `ResultRouteConfig` with the given result type.
#[proc_macro_derive(NavRoute, attributes(route))]
pub fn derive_nav_route(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut route_name = name.to_string();
    let mut result_type: Option<syn::Type> = None;

    for attr in &input.attrs {
        if attr.path().is_ident("route") {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("name") {
                    let s: syn::LitStr = meta.value()?.parse()?;
                    route_name = s.value();
                    Ok(())
                } else if meta.path.is_ident("result") {
                    let ty: syn::Type = meta.value()?.parse()?;
                    result_type = Some(ty);
                    Ok(())
                } else {
                    Ok(())
                }
            });
        }
    }

    let result_impl = if let Some(res) = result_type {
        quote! {
            impl #impl_generics ::gpui_transit::ResultRouteConfig for #name #ty_generics #where_clause {
                type Result = #res;
            }
        }
    } else {
        quote! {}
    };

    let expanded = quote! {
        #result_impl
    };

    TokenStream::from(expanded)
}
