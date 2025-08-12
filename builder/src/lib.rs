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

    let mut idents = Vec::new();
    let mut types = Vec::new();
    let mut build_fields = Vec::new();
    let mut setters = Vec::new();

    for f in fields.iter() {
        let field_ident = if let Some(ident) = f.ident.as_ref() {
            ident
        } else {
            return Err(Error::new_spanned(input, "没有匹配到结构体成员"));
        };
        let field_ty = &f.ty;

        if is_type_option(field_ty) {
            // 如果字段本身是 Option<T>
            let inner_ty = extract_option_type(field_ty)
                .ok_or_else(|| Error::new_spanned(field_ty, "无法提取 Option 内部类型"))?;
            idents.push(field_ident);
            types.push(quote! { #field_ty }); // 直接用原类型 Option<T>

            build_fields.push(quote! {
                #field_ident: self.#field_ident
            });

            setters.push(quote! {
                fn #field_ident(mut self, #field_ident: #inner_ty) -> Self {
                    self.#field_ident = std::option::Option::Some(#field_ident);
                    self
                }
            });
        } else {
            // 普通字段，builder 中用 Option<字段类型>
            idents.push(field_ident);
            types.push(quote! { std::option::Option<#field_ty> });

            build_fields.push(quote! {
                #field_ident: self.#field_ident.ok_or(format!("{} not set",stringify!(#field_ident)))?
            });

            setters.push(quote! {
                fn #field_ident(mut self, #field_ident: #field_ty) ->  Self {
                    self.#field_ident = std::option::Option::Some(#field_ident);
                    self
                }
            });
        }
    }

    Ok(quote! {
        struct #builder_name {
            #(#idents: #types),*
        }

        impl #ident {
            fn builder() -> #builder_name {
                #builder_name {
                    #(#idents: std::option::Option::None),*
                }
            }
        }

        impl #builder_name {
            #(#setters)*

            fn build(self) -> std::result::Result<#ident,std::boxed::Box<dyn std::error::Error>> {
                std::result::Result::Ok(#ident {
                    #(#build_fields),*
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
fn is_type_option(ty: &syn::Type) -> bool {
    matches!(
        ty,
        syn::Type::Path(tp)
            if tp.qself.is_none()
                && matches!(
                    tp.path.segments.last(),
                    Some(last_segment)
                        if last_segment.ident == "Option"
                            && matches!(last_segment.arguments, syn::PathArguments::AngleBracketed(_))
                )
    )
}
fn extract_option_type(ty: &syn::Type) -> Option<&syn::Type> {
    if is_type_option(ty) {
        if let syn::Type::Path(tp) = ty {
            if let Some(last_segment) = tp.path.segments.last() {
                if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                        return Some(inner_ty);
                    }
                }
            }
        }
    }
    None
}
