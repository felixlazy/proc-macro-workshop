use proc_macro::TokenStream;
use quote::quote;
use syn::{punctuated::Punctuated, DeriveInput, Error};

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
    let fields = extract_named_fields(input)?;
    let fields_named = fields
        .iter()
        .filter_map(|f| f.ident.as_ref().map(|ident| (ident, &f.ty)))
        .collect::<Vec<_>>();

    let (idents, tys): (Vec<_>, Vec<_>) = fields_named.iter().cloned().unzip();
    let functions = quote! {
            #(fn #idents(&mut self,#idents:#tys)->&mut Self{
                self.#idents=std::option::Option::Some(#idents);
                self
            })*
    };
    Ok(quote! {
        struct #builder_name{
            #(#idents:std::option::Option<#tys>),*
        }
        impl #ident{
            fn builder()->#builder_name{
                #builder_name{
                    #(#idents:std::option::Option::None),*
                }
            }
        }

        impl #builder_name{
            #functions
            fn build(&self)->std::option::Option<#ident>{
                std::option::Option::Some(#ident{
                    #(#idents: self.#idents.clone().unwrap()),*
                })
            }
        }

    })
}
fn extract_named_fields(
    input: &DeriveInput,
) -> Result<&Punctuated<syn::Field, syn::Token![,]>, Error> {
    if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(syn::FieldsNamed { named, .. }),
        ..
    }) = &input.data
    {
        Ok(named)
    } else {
        Err(Error::new_spanned(input, "没有匹配到结构体成员"))
    }
}
