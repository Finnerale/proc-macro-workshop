extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, punctuated::Punctuated, DeriveInput, Field};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let builder_name = format_ident!("{}Builder", name);

    let fields: Punctuated<Field, _> = {
        use syn::{Data::Struct, DataStruct, Fields::Named};
        if let Struct(DataStruct {
            fields: Named(fields),
            ..
        }) = input.data
        {
            fields.named
        } else {
            panic!("Builder derive only works for named structs")
        }
    };

    let builder_fields = fields.iter().map(|field| {
        let ident = field.ident.as_ref().unwrap();
        let mut ty = &field.ty;
        if let Some(oty) = get_optinal_type(ty) {
            ty = oty;
        }
        quote! {
            #ident: Option<#ty>,
        }
    });

    let builder_defaults = fields.iter().map(|field| {
        let ident = field.ident.as_ref().unwrap();
        quote! {
            #ident: None,
        }
    });

    let builder_setters = fields.iter().map(|field| {
        let ident = field.ident.as_ref().unwrap();
        let mut ty = &field.ty;
        if let Some(oty) = get_optinal_type(ty) {
            ty = oty;
        }
        quote! {
            pub fn #ident(&mut self, #ident: #ty) -> &mut Self {
                self.#ident = Some(#ident);
                self
            }
        }
    });

    let builder_checks = fields.iter().map(|field| {
        let ident = field.ident.as_ref().unwrap();
        if get_optinal_type(&field.ty).is_none() {
            quote! {
                if self.#ident.is_none() {
                    return Err(format!("{} has not been set.",
                            stringify!(#ident)).into());
                }
            }
        } else {
            TokenStream::new().into()
        }
    });

    let builder_unwraps = fields.iter().map(|field| {
        let ident = field.ident.as_ref().unwrap();
        if get_optinal_type(&field.ty).is_some() {
            quote! {
                #ident: self.#ident.take(),
            }
        } else {
            quote! {
                #ident: self.#ident.take().unwrap(),
            }
        }
    });

    let expanded = quote! {
        impl #name {
            pub fn builder() -> #builder_name {
                #builder_name {
                    #(#builder_defaults)*
                }
            }
        }

        pub struct #builder_name {
            #(#builder_fields)*
        }

        impl #builder_name {
            #(#builder_setters)*

            pub fn build(&mut self) -> Result<#name, Box<dyn std::error::Error>> {
                #(#builder_checks)*

                let command = #name {
                    #(#builder_unwraps)*
                };

                Ok(command)
            }
        }
    };

    TokenStream::from(expanded)
}

fn get_optinal_type(ty: &syn::Type) -> Option<&syn::Type> {
    use syn::AngleBracketedGenericArguments as ABGA;
    if let syn::Type::Path(syn::TypePath {
        qself: None,
        path: syn::Path { segments, .. },
    }) = ty
    {
        if let Some(syn::PathSegment {
            ident,
            arguments: syn::PathArguments::AngleBracketed(ABGA { args, .. }),
        }) = segments.first()
        {
            if ident == &format_ident!("Option") {
                if let Some(syn::GenericArgument::Type(oty)) = args.first() {
                    return Some(oty);
                }
            }
        }
    }
    None
}
