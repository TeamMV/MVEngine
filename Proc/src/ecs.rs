use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitInt, Ident};

pub fn generate_queries(input: TokenStream) -> TokenStream {
    let max_n = parse_macro_input!(input as LitInt).base10_parse::<usize>().unwrap();

    let methods = (1..=max_n).map(|n| {
        // Generics: C1, C2, ...
        let generics: Vec<Ident> = (1..=n)
            .map(|i| Ident::new(&format!("C{}", i), proc_macro2::Span::call_site()))
            .collect();

        // Method names
        let method_name = Ident::new(&format!("query{}", n), proc_macro2::Span::call_site());
        let method_name_mut = Ident::new(&format!("query{}_mut", n), proc_macro2::Span::call_site());

        // Rest-of-components fetch for immutable
        let rest_gets = if n > 1 {
            let rest: Vec<_> = generics.iter().skip(1).map(|g| {
                quote! {
                    let #g = self.get_component::<#g>(en)?;
                }
            }).collect();
            quote! { #(#rest)* }
        } else { quote! {} };

        // Tuple for immutable
        let tuple_refs: Vec<_> = generics.iter().map(|g| quote! { #g }).collect();

        // Rest-of-components fetch for mutable
        let rest_gets_mut = if n > 1 {
            let rest: Vec<_> = generics.iter().skip(1).map(|g| {
                quote! {
                    let #g = self.get_component_mut_bruh::<#g>(en)?;
                }
            }).collect();
            quote! { #(#rest)* }
        } else { quote! {} };

        // Tuple for mutable
        let tuple_refs_mut: Vec<_> = generics.iter().map(|g| quote! { #g }).collect();

        quote! {
            #[auto_enums::auto_enum(Iterator)]
            pub fn #method_name< #( #generics : Sized + 'static ),* >(
                &self
            ) -> impl Iterator<Item = (EntityId, ( #( & #tuple_refs ),* ))> + '_ {
                let t1 = std::any::TypeId::of::<C1>();
                if let Some(blob1) = self.components.get(&t1) {
                    blob1.get_all::<C1>().filter_map(|(idx, C1)| {
                        let en = self.get_entity_from_component_instance::<C1>(idx)?;
                        #rest_gets
                        Some((en, (#( #generics ),* )))
                    })
                } else {
                    std::iter::empty()
                }
            }

            #[auto_enums::auto_enum(Iterator)]
            pub fn #method_name_mut< #( #generics : Sized + 'static ),* >(
                &mut self
            ) -> impl Iterator<Item = (EntityId, ( #( &mut #tuple_refs_mut ),* ))> + '_ {
                let t1 = std::any::TypeId::of::<C1>();
                if let Some(blob1) = self.components.get(&t1) {
                    blob1.get_all_mut::<C1>().filter_map(|(idx, C1)| {
                        let en = self.get_entity_from_component_instance::<C1>(idx)?;
                        #rest_gets_mut
                        Some((en, (#( #generics ),* )))
                    })
                } else {
                    std::iter::empty()
                }
            }
        }
    });

    TokenStream::from(quote! {
        #( #methods )*
    })
}

pub fn generate_system_impls(input: TokenStream) -> TokenStream {
    let max_n = parse_macro_input!(input as LitInt).base10_parse::<usize>().unwrap();

    let impls = (1..=max_n).map(|n| {
        // Generics: C1, C2, ...
        let generics: Vec<Ident> = (1..=n)
            .map(|i| Ident::new(&format!("C{}", i), proc_macro2::Span::call_site()))
            .collect();

        // System tuple type: (C1, C2, ...)
        let tuple_type = if generics.len() == 1 {
            let ident = &generics[0];
            quote! { (#ident,) }
        } else {
            quote! { ( #( #generics ),* ) }
        };

        // query function name in storage: queryN / queryN_mut
        let query_fn = Ident::new(&format!("query{}", n), proc_macro2::Span::call_site());
        let query_fn_mut = Ident::new(&format!("query{}_mut", n), proc_macro2::Span::call_site());

        quote! {
            impl< #( #generics : Sized + 'static ),* > System<#tuple_type> {
                #[auto_enums::auto_enum(Iterator)]
                pub fn iter<'a>(&'a self, world: &'a World)
                    -> impl Iterator<Item=(EntityId, ( #( & #generics ),* ))> + 'a {
                    match world {
                        World::SparseSet(ssw) => ssw.storage.#query_fn::<#( #generics ),*>(),
                        World::ArchetypeWorld(atw) => std::iter::empty(),
                    }
                }

                #[auto_enums::auto_enum(Iterator)]
                pub fn iter_mut<'a>(&'a mut self, world: &'a mut World)
                    -> impl Iterator<Item=(EntityId, ( #( &mut #generics ),* ))> + 'a {
                    match world {
                        World::SparseSet(ssw) => ssw.storage.#query_fn_mut::<#( #generics ),*>(),
                        World::ArchetypeWorld(atw) => std::iter::empty(),
                    }
                }
            }
        }
    });

    TokenStream::from(quote! {
        #( #impls )*
    })
}