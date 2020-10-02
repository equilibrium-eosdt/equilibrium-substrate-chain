extern crate proc_macro;

use proc_macro::{Literal, TokenStream, TokenTree};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::parse_macro_input;
use syn::punctuated::Punctuated;
use syn::{Expr, Token};

struct Input {
    name: syn::Ident,
    args: Punctuated<Expr, Token![,]>,
}

impl Parse for Input {
    fn parse(input: ParseStream) -> syn::parse::Result<Self> {
        let name = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let args = Punctuated::<Expr, Token![,]>::parse_terminated(input)?;
        // let args = parser.parse(input);

        Ok(Input { name, args })
    }
}

fn tuple_read(name: syn::Ident, num: usize) -> syn::Expr {
    let num = Literal::usize_unsuffixed(num);
    let num: TokenTree = num.into();
    let num: TokenStream = num.into();
    let num: syn::Lit = syn::parse(num).unwrap();
    let gen = quote! {
        #name.#num
    };

    let tokens: TokenStream = gen.into();

    syn::parse(tokens).unwrap()
}

#[proc_macro]
pub fn tuple_to_vec(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as Input);
    let name = input.name;

    let vec_args: Punctuated<syn::Expr, Token![,]> = input
        .args
        .into_iter()
        .enumerate()
        .map(|(i, _)| tuple_read(name.clone(), i))
        .collect();

    let gen = quote! {
        vec!(#vec_args)
    };

    gen.into()
}
