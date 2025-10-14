use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, ItemStruct, Meta, PathArguments, Type};

pub fn from_json(input: TokenStream) -> TokenStream {
    let struct_item = parse_macro_input!(input as ItemStruct);

    let ident = struct_item.ident;

    let mut field_init_ts = quote! {};
    let mut self_init_ts = quote! {};
    for field in struct_item.fields {
        if let Some(field_ident) = field.ident {
            let field_name_str = field_ident.to_string();
            let ty = field.ty.clone();
            let uses_default = field.attrs
                .iter()
                .filter_map(|attr| {
                    if attr.path().is_ident("default_value") {
                        match &attr.meta {
                            Meta::List(meta_list) => {
                                let tokens = meta_list.tokens.clone();
                                Some(tokens)
                            }
                            _ => None,
                        }
                    } else {
                        None
                    }
                })
                .next();


            let is_option = match &field.ty {
                Type::Path(type_path) => {
                    type_path
                        .path
                        .segments
                        .last()
                        .map(|seg| seg.ident == "Option")
                        .unwrap_or(false)
                }
                _ => false,
            };

            if is_option {
                // Extract inner type of Option<T>
                let inner_ty = match &field.ty {
                    Type::Path(type_path) => {
                        if let Some(seg) = type_path.path.segments.last() {
                            if let PathArguments::AngleBracketed(args) = &seg.arguments {
                                if let Some(syn::GenericArgument::Type(inner)) = args.args.first() {
                                    inner.clone()
                                } else {
                                    ty.clone()
                                }
                            } else {
                                ty.clone()
                            }
                        } else {
                            ty.clone()
                        }
                    }
                    _ => ty.clone(),
                };

                field_init_ts.extend(quote! {
                    let #field_ident = if let Some(value) = obj.get(#field_name_str) {
                        Some(<#inner_ty as FromJsonTrait>::from_json(value)?)
                    } else {
                        None
                    };
                });
            } else {
                if let Some(def) = uses_default {
                    field_init_ts.extend(quote! {
                        let #field_ident = if let Some(value) = obj.get(#field_name_str) {
                            <#ty as FromJsonTrait>::from_json(value)?
                        } else {
                            #def
                        };
                    });
                } else {
                    field_init_ts.extend(quote! {
                    let #field_ident = obj
                        .get(#field_name_str)
                        .ok_or(FromJsonError::NoSuchField(#field_name_str.to_string()))?;
                    let #field_ident = <#ty as FromJsonTrait>::from_json(#field_ident)?;
                });
                }
            }

            self_init_ts.extend(quote! {
                #field_ident,
            });
        }
    }

    let ts = quote! {
        impl FromJsonTrait for #ident {
            fn from_json(json: &JsonElement) -> Result<Self, FromJsonError>
            where
                Self: Sized
            {
                let obj = Self::illegal_conversion(json.as_object())?;
                #field_init_ts

                Ok(Self { #self_init_ts })
            }
        }
    };

    ts.into()
}
