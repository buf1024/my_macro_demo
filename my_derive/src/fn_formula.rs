use proc_macro2::TokenStream;
use syn::{parse::Parse, parse_quote, punctuated::Punctuated, LitInt, Result, Token};

mod kw {
    use syn::custom_keyword;
    custom_keyword!(x);
    custom_keyword!(y);
}

pub(crate) struct FormulaArgs {
    formula: Vec<Formula>,
}

impl Parse for FormulaArgs {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let attrs = Punctuated::<Formula, Token![,]>::parse_terminated(input)?;
        let formula: Vec<_> = attrs.into_iter().collect();
        if formula.len() != 2 {
            panic!("require two formula")
        }
        Ok(FormulaArgs { formula })
    }
}

#[derive(Default)]
pub(crate) struct Formula {
    x: i32,
    y: i32,
    rs: i32,
}

impl Parse for Formula {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();

        let x = if lookahead.peek(LitInt) {
            let x_lit: LitInt = input.parse()?;
            let x: i32 = x_lit.to_string().parse().unwrap();
            let _: Token![*] = input.parse()?;
            let _: kw::x = input.parse()?;

            x
        } else {
            return Err(lookahead.error());
        };

        let r1: Result<Token![+]> = input.parse();
        let r2: Result<Token![-]> = input.parse();
        let factor = match (r1, r2) {
            (Ok(_), Err(_)) => 1,
            (Err(_), Ok(_)) => -1,
            (_, _) => return Err(lookahead.error()),
        };

        // let factor = if lookahead.peek(Token![+]) {
        //     let _: Token![+] = input.parse()?;
        //     1
        // } else if lookahead.peek(Token![-]) {
        //     let _: Token![-] = input.parse()?;
        //     -1
        // } else {
        //     return Err(lookahead.error());
        // };

        let y = if lookahead.peek(LitInt) {
            let y_lit: LitInt = input.parse()?;
            let y: i32 = y_lit.to_string().parse().unwrap();
            let _: Token![*] = input.parse()?;
            let _: kw::y = input.parse()?;
            y * factor
        } else {
            return Err(lookahead.error());
        };

        let _: Token![=] = input.parse()?;

        let rs_lit: LitInt = input.parse()?;
        let rs: i32 = rs_lit.to_string().parse().unwrap();

        if x == 0 && y == 0 && rs != 0 {
            return Err(syn::Error::new_spanned(rs_lit, "invalid equal"));
        }

        Ok(Self { x, y, rs })
    }
}

pub(crate) fn expand_formula(formula: FormulaArgs) -> Result<TokenStream> {
    let f1 = formula.formula.get(0).unwrap();
    let f2 = formula.formula.get(1).unwrap();
    match (f1.x, f1.y) {
        (0, 0) => match (f2.x, f2.y) {
            (0, 0) => return Ok(parse_quote!((0.0, 0.0))),
            (0, _) => {
                let y = f2.rs as f32 / f2.y as f32;
                return Ok(parse_quote!((0.0, #y)));
            }
            (_, 0) => {
                let x = f2.rs as f32 / f2.x as f32;

                return Ok(parse_quote!((#x, 0.0)));
            }
            (_, _) => panic!("no solution"),
        },
        (0, _) => match (f2.x, f2.y) {
            (0, 0) => {
                let y = f1.rs as f32 / f1.y as f32;
                return Ok(parse_quote!((0.0, #y)));
            }
            (0, _) => {
                let y1 = f1.rs as f32 / f1.y as f32;
                let y2 = f2.rs as f32 / f2.y as f32;
                if y1 != y2 {
                    panic!("no solution")
                }

                return Ok(parse_quote!((0.0, #y1)));
            }
            (_, 0) => {
                let x = f2.rs as f32 / f2.x as f32;
                let y = f1.rs as f32 / f1.y as f32;

                return Ok(parse_quote!((#x, #y)));
            }
            (_, _) => {
                let y = f1.rs as f32 / f1.y as f32;
                let x = (f2.rs as f32 - y * f2.y as f32) as f32 / f2.x as f32;

                return Ok(parse_quote!((#x, #y)));
            }
        },
        (_, 0) => match (f2.x, f2.y) {
            (0, 0) => {
                let x = f1.rs as f32 / f1.y as f32;
                return Ok(parse_quote!((#x, 0)));
            }
            (0, _) => {
                let x = f1.rs as f32 / f1.x as f32;
                let y = f2.rs as f32 / f2.y as f32;
                return Ok(parse_quote!((#x, #y)));
            }
            (_, 0) => {
                let x1 = f1.rs as f32 / f1.x as f32;
                let x2 = f2.rs as f32 / f2.x as f32;
                if x1 != x2 {
                    panic!("no solution")
                }
                return Ok(parse_quote!((#x1, 0.0)));
            }
            (_, _) => {
                let x = f1.rs as f32 / f1.x as f32;
                let y = (f2.rs as f32 - x * f2.x as f32) as f32 / f2.y as f32;

                return Ok(parse_quote!((#x, #y)));
            }
        },
        (_, _) => match (f2.x, f2.y) {
            (0, 0) => panic!("no solution"),
            (0, _) => {
                let y = f2.rs as f32 / f2.y as f32;
                let x = (f1.rs as f32 - y * f1.y as f32) as f32 / f1.x as f32;

                return Ok(parse_quote!((#x, #y)));
            }
            (_, 0) => {
                let x = f2.rs as f32 / f2.x as f32;
                let y = (f1.rs as f32 - x * f1.x as f32) as f32 / f1.y as f32;

                return Ok(parse_quote!((#x, #y)));
            }
            (_, _) => {
                if f1.x == f2.x && f1.y == f2.y {
                    panic!("no solution")
                }
                let factor = f1.x as f32 / f2.x as f32;

                let y = (f2.rs as f32 * factor - f1.rs as f32) as f32
                    / (f2.y as f32 * factor - f1.y as f32);
                let x = (f2.rs as f32 - f2.y as f32 * y) / f2.x as f32;

                return Ok(parse_quote!((#x, #y)));
            }
        },
    }
}
