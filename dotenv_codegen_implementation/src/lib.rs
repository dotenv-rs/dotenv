extern crate proc_macro;

use std::env::{self, VarError};

use proc_macro::TokenStream;
use proc_macro_hack::proc_macro_hack;
use quote::quote;
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::Token;

#[proc_macro_hack]
pub fn dotenv(input: TokenStream) -> TokenStream {
    if let Err(err) = dotenv::dotenv() {
        let err_msg = format!("Error loading .env file: {}", err);
        return quote! {
            compile_error!(#err_msg);
        }
        .into();
    }

    // Either everything was fine, or we didn't find an .env file (which we ignore)
    let (var_name, second_value) = expand_env(input);

    let err_msg = match second_value {
        Some(e) => e,
        None => format!("environment variable `{}` not defined", var_name),
    };

    match env::var(var_name) {
        Ok(val) => quote!(#val).into(),
        Err(VarError::NotPresent) | Err(VarError::NotUnicode(_)) => panic!("{}", err_msg),
    }
}

#[proc_macro_hack]
pub fn dotenv_or_default(input: TokenStream) -> TokenStream {
    if let Err(err) = dotenv::dotenv() {
        let err_msg = format!("Error loading .env file: {}", err);
        return quote! {
            compile_error!(#err_msg);
        }
        .into();
    }

    // Either everything was fine, or we didn't find an .env file (which we ignore)
    let (var_name, second_value) = expand_env(input);

    match second_value {
        Some(default) => match env::var(var_name) {
            Ok(val) => quote!(#val).into(),
            Err(VarError::NotPresent) | Err(VarError::NotUnicode(_)) => quote!(#default).into(),
        },
        None => {
            let err_msg = format!("Missing default value for: {}", var_name);
            (quote! {
                compile_error!(#err_msg)
            })
            .into()
        }
    }
}

fn expand_env(input_raw: TokenStream) -> (String, Option<String>) {
    let args = <Punctuated<syn::LitStr, Token![,]>>::parse_terminated
        .parse(input_raw)
        .expect("expected macro to be called with a comma-separated list of string literals");

    let mut iter = args.iter();

    let var_name = match iter.next() {
        Some(s) => s.value(),
        None => panic!("expected 1 or 2 arguments, found none"),
    };

    let second_value = match iter.next() {
        Some(lit) => Some(lit.value()),
        None => None,
    };

    if iter.next().is_some() {
        panic!("expected 1 or 2 arguments, found 3 or more");
    }

    (var_name, second_value)
}
