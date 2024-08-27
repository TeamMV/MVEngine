use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Type};

pub fn graphics_item(attrib: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let ident = input.ident.clone();

    let (i, t, w) = input.generics.split_for_impl();

    let s = attrib.to_string();

    let vk = match &input.data {
        Data::Enum(data) => get_field(&data.variants[0].fields),
        _ => unreachable!(),
    };

    match s.as_str() {
        "ref" => quote! {
            #input

            impl #i #ident #t #w {
                pub fn as_vulkan(&self) -> &#vk {
                    #[allow(irrefutable_let_patterns)]
                    let #ident::Vulkan(item) = self else { unreachable!() };
                    item
                }

                pub fn as_vulkan_mut(&mut self) -> &mut #vk {
                    #[allow(irrefutable_let_patterns)]
                    let #ident::Vulkan(item) = self else { unreachable!() };
                    item
                }

                pub fn into_vulkan(self) -> #vk {
                    #[allow(irrefutable_let_patterns)]
                    let #ident::Vulkan(item) = self else { unreachable!() };
                    item
                }
            }
        },
        "clone" => quote! {
            #input

            impl #i #ident #t #w {
                #[allow(irrefutable_let_patterns)]
                pub fn as_vulkan(&self) -> #vk {
                    let #ident::Vulkan(item) = self else { unreachable!() };
                    item.clone()
                }

                pub fn into_vulkan(self) -> #vk {
                    #[allow(irrefutable_let_patterns)]
                    let #ident::Vulkan(item) = self else { unreachable!() };
                    item
                }
            }
        },
        "copy" => quote! {
            #input

            impl #i #ident #t #w {
                #[allow(irrefutable_let_patterns)]
                pub fn as_vulkan(&self) -> #vk {
                    let #ident::Vulkan(item) = self else { unreachable!() };
                    *item
                }

                pub fn into_vulkan(self) -> #vk {
                    #[allow(irrefutable_let_patterns)]
                    let #ident::Vulkan(item) = self else { unreachable!() };
                    item
                }
            }
        },
        _ => unreachable!(),
    }
    .into()
}

fn get_field(fields: &Fields) -> Type {
    match fields {
        Fields::Unnamed(fields) => fields.unnamed[0].ty.clone(),
        _ => unreachable!(),
    }
}
