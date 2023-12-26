mod derive_getters;
use derive_getters::expand_getters;

mod attr_retry;
use attr_retry::{expand_retry, Args};

mod fn_formula;
use fn_formula::{expand_formula, FormulaArgs};

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, ItemFn};

#[proc_macro_derive(Getters, attributes(getter))]
pub fn getters(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let token_stream = expand_getters(input);
    token_stream
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

#[proc_macro_attribute]
pub fn retry(attr: TokenStream, input: TokenStream) -> TokenStream {
    let item_fn = parse_macro_input!(input as ItemFn);

    let args = parse_macro_input!(attr as Args);

    let token_stream = expand_retry(args.attr, item_fn);
    token_stream
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

#[proc_macro]
pub fn formula(input: TokenStream) -> TokenStream {
    let my_formula = parse_macro_input!(input as FormulaArgs);

    let token_stream = expand_formula(my_formula);
    token_stream
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}
