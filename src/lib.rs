//! Async trait prototype using the desugarization described in [RFC 3185 Static Async Fn in Traits](https://rust-lang.github.io/rfcs/3185-static-async-fn-in-trait.html#equivalent-desugaring).
//!
//! It should be faster than [async-trait](https://crates.io/crates/async-trait) because it doesn't use allocations on every invocation and type erasure.
//!
//! Requires these feature flags and a **nightly compiler**:
//! - `#![feature(generic_associated_types)]`
//! - `#![feature(type_alias_impl_trait)]`
//!
//! ## Example
//! ```
//! #![feature(generic_associated_types)]
//! #![feature(type_alias_impl_trait)]
//! # use std::time::Duration;
//! # use tokio::time::sleep;
//! # use tokio::runtime::Builder;
//! # let runtime = Builder::new_current_thread().build().unwrap();
//! use async_trait_proto::async_trait_proto;
//! struct Foo;
//!
//! #[async_trait_proto]
//! trait Bar {
//!     async fn wait(&self);
//! }
//!
//! #[async_trait_proto]
//! impl Bar for Foo {
//!     async fn wait(&self) {
//!         sleep(Duration::from_secs(10)).await;
//!     }
//! }
//! # runtime.block_on(async move {
//! # });
//! ```

extern crate proc_macro;

#[rustversion::stable]
compile_error!("macro only works on nightly toolchain, since the nightly features `generic_associated_types` and `type_alias_impl_trait` are necessary");

#[rustversion::beta]
compile_error!("macro only works on nightly toolchain, since the nightly features `generic_associated_types` and `type_alias_impl_trait` are necessary");

use proc_macro::TokenStream;

use quote::{quote, ToTokens};
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, FnArg, GenericParam, ImplItem, ImplItemMethod, Item, ItemImpl, ItemTrait,
    Lifetime, LifetimeDef, ReturnType, TraitItem, TraitItemMethod,
};

#[proc_macro_attribute]
pub fn async_trait_proto(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Item);

    match input {
        Item::Trait(trait_item) => generate_trait(trait_item),
        Item::Impl(implt) => generate_impl(implt),
        _ => panic!("Invalid"),
    }
}

fn generate_impl(input: ItemImpl) -> TokenStream {
    let mut items = Vec::new();
    for item in input.items {
        match item {
            ImplItem::Const(c) => items.push(c.into_token_stream()),
            ImplItem::Method(m) => generate_method_impl(m, &mut items),
            ImplItem::Type(t) => items.push(t.into_token_stream()),
            ImplItem::Macro(m) => items.push(m.into_token_stream()),
            ImplItem::Verbatim(v) => items.push(v.into_token_stream()),
            _ => unimplemented!(),
        }
    }

    let unsafty = input.unsafety;
    let generics = input.generics;
    let (_, tr, _) = input.trait_.unwrap();
    let s = input.self_ty;

    quote! {
        #unsafty impl #generics #tr for #s {
            #(#items)*
        }
    }
    .into()
}

fn generate_trait(input: ItemTrait) -> TokenStream {
    let mut items = Vec::new();

    for item in input.items {
        match item {
            TraitItem::Type(t) => items.push(t.into_token_stream()),
            TraitItem::Const(c) => items.push(c.into_token_stream()),
            TraitItem::Macro(m) => items.push(m.into_token_stream()),
            TraitItem::Verbatim(v) => items.push(v),
            TraitItem::Method(m) => generate_method(m, &mut items),
            _ => unimplemented!(),
        }
    }

    let visiblity = input.vis;
    let trait_ident = input.ident;
    let generics = input.generics;
    let m = input.supertraits;

    quote! {
        #visiblity trait #trait_ident #generics: #m {
            #(#items)*
        }

    }
    .into()
}

fn generate_method_impl(m: ImplItemMethod, items: &mut Vec<proc_macro2::TokenStream>) {
    if m.sig.asyncness.is_none() {
        items.push(m.into_token_stream());
        return;
    }

    let mut self_lifetime = false;

    for arg in m.sig.inputs.iter() {
        if let FnArg::Receiver(ref r) = arg {
            if let Some((_, None)) = &r.reference {
                self_lifetime = true;
            }
        }
    }
    let output_type = match m.sig.output {
        ReturnType::Default => quote! { () },
        ReturnType::Type(_, ref t) => t.into_token_stream(),
    };
    let ass_typ_name = associated_type_impl(&m);
    let associated_type = quote! {
        type #ass_typ_name<'a> =  impl core::future::Future<Output = #output_type> + 'a
        where
            Self: 'a;
    };

    let mut generics = m.sig.generics;
    let ident = m.sig.ident;
    let inputs = m.sig.inputs;
    let unsafety = m.sig.unsafety;
    let block = m.block;

    let function = if self_lifetime {
        quote! {
            #unsafety fn #ident(#inputs) -> Self::#ass_typ_name<'_> {
                  async move #block
              }
        }
    } else {
        generics
            .params
            .push(GenericParam::Lifetime(LifetimeDef::new(Lifetime::new(
                "'async_trait",
                generics.span(),
            ))));
        quote! {
          #unsafety fn #ident<'async_trait>(#inputs) -> Self::#ass_typ_name<'async_trait> {
            async move #block
            }
        }
    };

    items.push(associated_type);
    items.push(function);
}

fn generate_method(m: TraitItemMethod, items: &mut Vec<proc_macro2::TokenStream>) {
    if m.sig.asyncness.is_none() {
        items.push(m.into_token_stream());
        return;
    }

    let mut self_lifetime = false;

    for arg in m.sig.inputs.iter() {
        if let FnArg::Receiver(ref r) = arg {
            if let Some((_, None)) = &r.reference {
                self_lifetime = true;
            }
        }
    }
    let output_type = match m.sig.output {
        ReturnType::Default => quote! { () },
        ReturnType::Type(_, ref t) => t.into_token_stream(),
    };

    let ass_typ_name = associated_type(&m);
    let associated_type = quote! {
        type #ass_typ_name<'a>: core::future::Future<Output = #output_type>
        where
            Self: 'a;
    };

    let mut generics = m.sig.generics;
    let ident = m.sig.ident;
    let inputs = m.sig.inputs;
    let unsafety = m.sig.unsafety;

    let function = if self_lifetime {
        quote! {
            #unsafety fn #ident(#inputs) -> Self::#ass_typ_name<'_>;
        }
    } else {
        generics
            .params
            .push(GenericParam::Lifetime(LifetimeDef::new(Lifetime::new(
                "'async_trait",
                generics.span(),
            ))));
        quote! {
          #unsafety fn #ident<'async_trait>(#inputs) -> Self::#ass_typ_name<'async_trait>;
        }
    };

    items.push(associated_type);
    items.push(function);
}

fn associated_type_impl(m: &ImplItemMethod) -> proc_macro2::TokenStream {
    let mut associated_type_name: String = m
        .sig
        .ident
        .to_string()
        .chars()
        .enumerate()
        .map(|(idx, e)| if idx == 0 { e.to_ascii_uppercase() } else { e })
        .collect();
    associated_type_name.push_str("Fut");
    associated_type_name.parse().unwrap()
}

fn associated_type(m: &TraitItemMethod) -> proc_macro2::TokenStream {
    let mut associated_type_name: String = m
        .sig
        .ident
        .to_string()
        .chars()
        .enumerate()
        .map(|(idx, e)| if idx == 0 { e.to_ascii_uppercase() } else { e })
        .collect();
    associated_type_name.push_str("Fut");
    associated_type_name.parse().unwrap()
}
