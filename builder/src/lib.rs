use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Error};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    match parse(&input) {
        Ok(token_stream) => token_stream.into(),
        Err(e) => e.to_compile_error().into(),
    }
}
fn parse(input: &DeriveInput) -> Result<proc_macro2::TokenStream, Error> {
    let ident = &input.ident;
    let builder_name = quote::format_ident!("{ident}Builder");
    Ok(quote! {
        struct #builder_name{}
        impl #ident{
            fn builder()->#builder_name{
                #builder_name{

                }
            }
        }
    })
}
