use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse::Parse, parse_quote, punctuated::Punctuated, Data, DataStruct, DeriveInput, Fields,
    Ident, LitStr, Result, Token, Type, TypeReference, Visibility,
};

mod kw {
    use syn::custom_keyword;
    custom_keyword!(name);
    custom_keyword!(vis);
}

#[derive(Default)]
struct GetterMeta {
    name: Option<Ident>,
    vis: Option<Visibility>,
}

impl GetterMeta {
    fn merge(self, other: GetterMeta) -> Result<Self> {
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
            name: either(self.name, other.name)?,
            vis: either(self.vis, other.vis)?,
        })
    }
}

impl Parse for GetterMeta {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::name) {
            let _: kw::name = input.parse()?;
            let _: Token![=] = input.parse()?;

            let name: Ident = if input.peek(LitStr) {
                let sl: LitStr = input.parse()?;
                let value = sl.value();

                format_ident!("{}", value.trim())
            } else {
                input.parse()?
            };

            Ok(Self {
                name: Some(name),
                vis: None,
            })
        } else if lookahead.peek(kw::vis) {
            let _: kw::vis = input.parse()?;
            let _: Token![=] = input.parse()?;

            let vis = input.parse()?;

            Ok(Self {
                name: None,
                vis: Some(vis),
            })
        } else {
            Err(lookahead.error())
        }
    }
}

pub(crate) fn expand_getters(input: DeriveInput) -> Result<TokenStream> {
    let fields = match input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => fields.named,
        _ => panic!("only allow struct filed"),
    };

    let getters = fields
        .into_iter()
        .map(|f| {
            let meta: GetterMeta = f
                .attrs
                .iter()
                .filter(|attr| attr.path().is_ident("getter"))
                .try_fold(GetterMeta::default(), |meta, attr| {
                    let list: Punctuated<GetterMeta, Token![,]> =
                        attr.parse_args_with(Punctuated::parse_terminated)?;

                    list.into_iter().try_fold(meta, GetterMeta::merge)
                })?;

            let doc_str: String = f
                .attrs
                .iter()
                .filter(|attr| attr.path().is_ident("doc"))
                .try_fold(String::new(), |acc, attr| {
                    let mnv = match &attr.meta {
                        syn::Meta::NameValue(mnv) => mnv,
                        _ => return Err(syn::Error::new_spanned(attr, "expect name value!")),
                    };
                    let doc_str = match &mnv.value {
                        syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(lit),
                            ..
                        }) => lit.value(),

                        _ => return Err(syn::Error::new_spanned(attr, "expect string literal!")),
                    };
                    Ok(format!("{}\n{}", acc, doc_str))
                })?;

            
            let doc_str = doc_str.trim();

            let visibility = meta.vis.unwrap_or_else(|| parse_quote! { pub });

            let filed_name = f.ident.unwrap();
            let filed_type = match f.ty {
                Type::Reference(
                    r @ TypeReference {
                        mutability: None, ..
                    },
                ) => quote!(#r),
                typ => quote!(&#typ),
            };
            let fn_name = meta
                .name
                .unwrap_or_else(|| format_ident!("get_{}", filed_name));

            let desc_fn = if !doc_str.is_empty() {
                let desc_name = format_ident!("{}_desc", fn_name);
                quote!(
                    pub fn #desc_name(&self) -> &'static str {
                        #doc_str
                    }
                )
            } else {
                TokenStream::new()
            };

            Ok(quote!(
                #visibility fn #fn_name(&self) -> #filed_type {
                    &self.#filed_name
                }

                #desc_fn
            ))
        })
        .collect::<Result<TokenStream>>()?;

    let st_name = input.ident;

    let (impl_generic, type_generic, where_clause) = input.generics.split_for_impl();

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generic #st_name #type_generic #where_clause {
            pub fn hello(&self) {
                println!("hello!");
            }
            #getters
        }
    })
}
