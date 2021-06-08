extern crate proc_macro;
use std::sync::atomic::{AtomicU32, Ordering};

use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Index, parse_macro_input};

#[proc_macro_derive(CompactTypeId)]
pub fn derive_compact_type_id(input: TokenStream) -> TokenStream {
    static COUNTER: AtomicU32 = AtomicU32::new(1); // We have to be careful where we start because of hard-coded compact type ids.

    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);

    let expanded = quote! {
        unsafe impl #impl_generics crate::gc::CompactTypeId for #name #ty_generics #where_clause {
            #[inline]
            fn compact_type_id() -> u32 {
                #id
            }
        }
        impl crate::gc::CheckTypeId<#id> for () {}
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(Trace)]
pub fn derive_trace(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let (trace_inner, finalize_inner) = match input.data {
        Data::Struct(data) => {
            let fields: Vec<_> = data.fields.iter().enumerate().map(|(i, field)| {
                if let Some(ident) = &field.ident {
                    quote!(self.#ident)
                } else {
                    let i = Index::from(i);
                    quote!(#i)
                }
            }).collect();
            (
                quote! {
                    #(
                        crate::gc::Trace::trace(&mut #fields);
                    )*
                },
                quote! {
                    #(
                        crate::gc::Trace::finalize(&mut #fields);
                    )*
                }
            )
        }
        Data::Enum(_) => todo!("deriving `Trace` on enums isn't implemented yet"),
        Data::Union(_) => panic!("cannot derive `Trace` on a union"),
    };

    let expanded = quote! {
        unsafe impl #impl_generics crate::gc::Trace for #name #ty_generics #where_clause {
            unsafe fn trace(&mut self) {
                #trace_inner
            }
            unsafe fn finalize(&mut self) {
                #finalize_inner
            }
        }
    };

    TokenStream::from(expanded)
}
