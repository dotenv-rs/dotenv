#[macro_use]
extern crate proc_macro_hack;
#[macro_use]
extern crate quote;

extern crate dotenv;
extern crate proc_macro;
extern crate proc_macro2;

#[macro_use]
extern crate syn;

use std::env;

use syn::punctuated::Punctuated;
use syn::parse::Parser;
use proc_macro::TokenStream;

#[proc_macro_hack]
pub fn dotenv(input: TokenStream) -> TokenStream {
    if let Err(err) = dotenv::dotenv() {
        if let dotenv::Error::LineParse(ref line) = err {
            panic!("Error parsing .env file: {}", line);
        } else {
            panic!("Error loading .env file: {}", err);
        }
    }

    // Either everything was fine, or we didn't find an .env file (which we ignore)
    expand_env(input)
}

fn expand_env(input_raw: TokenStream) -> TokenStream {

    let args = <Punctuated<syn::LitStr, Token![,]>>::parse_terminated.parse(input_raw)
        .expect("expected macro to be called with a comma-separated list of string literals");

    let mut iter = args.iter();

    let var_name = match iter.next() {
        Some(s) => s.value(),
        None => panic!("expected 1 or 2 arguments, found none"),
    };

    let err_msg = match iter.next() {
        Some(lit) => lit.value(),
        None => format!("environment variable `{}` not defined", var_name).into(),
    };

    if iter.next().is_some() {
        panic!("expected 1 or 2 arguments, found 3 or more");
    }

    match env::var(var_name) {
        Ok(val) => quote!(#val).into(),
        Err(_) => panic!("{}", err_msg),
    }
}
