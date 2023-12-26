use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse::Parse, parse_quote, punctuated::Punctuated, token::Comma, Expr, ItemFn, LitInt, Result,
    Signature, Token,
};

mod kw {
    use syn::custom_keyword;
    custom_keyword!(times);
    custom_keyword!(timeout);
}

pub(crate) struct Args {
    pub attr: RetryAttr,
}

impl Parse for Args {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let attrs = Punctuated::<RetryAttr, Token![,]>::parse_terminated(input)?;
        let attr = attrs
            .into_iter()
            .try_fold(RetryAttr::default(), RetryAttr::merge)?;
        Ok(Args { attr })
    }
}

#[derive(Default)]
pub(crate) struct RetryAttr {
    times: Option<LitInt>,
    timeout: Option<LitInt>,
}

impl RetryAttr {
    fn merge(self, other: RetryAttr) -> Result<Self> {
        fn either<T: ToTokens>(a: Option<T>, b: Option<T>) -> syn::Result<Option<T>> {
            match (a, b) {
                (None, None) => Ok(None),
                (Some(val), None) | (None, Some(val)) => Ok(Some(val)),
                (Some(a), Some(b)) => {
                    let mut error = syn::Error::new_spanned(a, "redundant attribute argument");
                    error.combine(syn::Error::new_spanned(b, "note: first one here"));
                    Err(error)
                }
            }
        }

        Ok(Self {
            times: either(self.times, other.times)?,
            timeout: either(self.timeout, other.timeout)?,
        })
    }
}

impl Parse for RetryAttr {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::times) {
            let _: kw::times = input.parse()?;
            let _: Token![=] = input.parse()?;

            let times: LitInt = if input.peek(LitInt) {
                input.parse()?
            } else {
                return Err(lookahead.error());
            };

            Ok(Self {
                times: Some(times),
                timeout: None,
            })
        } else if lookahead.peek(kw::timeout) {
            let _: kw::timeout = input.parse()?;
            let _: Token![=] = input.parse()?;

            let timeout: LitInt = if input.peek(LitInt) {
                input.parse()?
            } else {
                return Err(lookahead.error());
            };

            Ok(Self {
                times: None,
                timeout: Some(timeout),
            })
        } else {
            Err(lookahead.error())
        }
    }
}

pub(crate) fn expand_retry(attr: RetryAttr, item_fn: ItemFn) -> Result<TokenStream> {
    let timeout = if let Some(to) = attr.timeout {
        quote!(#to)
    } else {
        parse_quote!(60)
    };
    let times = if let Some(times) = attr.times {
        quote!(#times)
    } else {
        parse_quote!(3)
    };

    let new_fn_name = format_ident!("__new_{}", item_fn.sig.ident);

    let old_sig = Signature {
        ident: new_fn_name.clone(),
        ..item_fn.sig.clone()
    };
    let old_fn = ItemFn {
        sig: old_sig,
        ..item_fn.clone()
    };

    let idents = item_fn.sig.inputs.iter().filter_map(|args| {
        if let syn::FnArg::Typed(pat_type) = args {
            if let syn::Pat::Ident(pat_ident) = *pat_type.pat.clone() {
                return Some(pat_ident.ident);
            }
        }
        None
    });
    let mut punctuated: Punctuated<syn::Ident, Comma> = Punctuated::new();
    idents.for_each(|ident| punctuated.push(ident));

    let args: Expr = parse_quote!((#punctuated));

    let new_fn = ItemFn {
        block: parse_quote!(
            {
                #old_fn

                for _ in 0..#times {
                    // assume failed
                    #new_fn_name #args;
                }
                for _ in 0..#timeout {
                    // assume failed
                    #new_fn_name #args;
                }
                 #new_fn_name #args 
            }
        ),
        ..item_fn
    };

    let qt = quote!(
        #new_fn
    );

    Ok(qt)
}
