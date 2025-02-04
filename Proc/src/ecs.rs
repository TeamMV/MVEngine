use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitInt, Ident};

pub fn generate_get_components(input: TokenStream) -> TokenStream {
    let num_generics = parse_macro_input!(input as LitInt).base10_parse::<usize>().unwrap();

    let methods = (1..=num_generics).map(|n| {
        // Create identifiers for generics (C1, C2, ...)
        let generics: Vec<Ident> = (1..=n)
            .map(|i| Ident::new(&format!("C{}", i), proc_macro2::Span::call_site()))
            .collect();

        // Create identifiers for component variables (c1, c2, ...)
        let c_vars: Vec<Ident> = (1..=n)
            .map(|i| Ident::new(&format!("c{}", i), proc_macro2::Span::call_site()))
            .collect();

        // Create identifiers for indices (idx1, idx2, ...)
        let idx_vars: Vec<Ident> = (1..=n)
            .map(|i| Ident::new(&format!("idx{}", i), proc_macro2::Span::call_site()))
            .collect();

        // Immutable method
        let fetch_components = quote! {
            #(
                let #c_vars = self.components.get(&TypeId::of::<#generics>())?.get_all::<#generics>();
            )*
        };

        let nested_conditions = (0..n).rev().fold(quote! {
            out.push((*en, #( *#c_vars ),*));
        }, |acc, i| {
            let idx_var = &idx_vars[i];
            let c_var = &c_vars[i];
            let generic = &generics[i];
            quote! {
                if let Some(#idx_var) = map.get(&TypeId::of::<#generic>()) {
                    if let Some(#c_var) = #c_var.get(*#idx_var as usize) {
                        #acc
                    }
                }
            }
        });

        let method_name = Ident::new(&format!("get_components{}", n), proc_macro2::Span::call_site());
        let immutable_method = quote! {
            pub fn #method_name< #( #generics : Sized + 'static ),* >(&self)
                -> Option<Vec<(EntityType, #( & #generics ),*)>> {

                #fetch_components

                let mut out = vec![];

                for (en, map) in self.entity_components.iter() {
                    #nested_conditions
                }

                Some(out)
            }
        };

        // Mutable method
        let fetch_components_mut = quote! {
            #(
                let mut #c_vars = self.components.get(&TypeId::of::<#generics>())?.get_all::<#generics>();
            )*
        };

        let nested_conditions_mut = (0..n).rev().fold(quote! {
            out.push((*en, #( unsafe { (*#c_vars as *const #generics as *mut #generics).as_mut().unwrap() } ),*));
        }, |acc, i| {
            let idx_var = &idx_vars[i];
            let c_var = &c_vars[i];
            let generic = &generics[i];
            quote! {
                if let Some(#idx_var) = map.get(&TypeId::of::<#generic>()) {
                    if let Some(#c_var) = #c_var.get_mut(*#idx_var as usize) {
                        #acc
                    }
                }
            }
        });

        let method_name_mut = Ident::new(&format!("get_components{}_mut", n), proc_macro2::Span::call_site());
        let mutable_method = quote! {
            pub fn #method_name_mut< #( #generics : Sized + 'static ),* >(&mut self)
                -> Option<Vec<(EntityType, #( &mut #generics ),*)>> {

                #fetch_components_mut

                let mut out = vec![];

                for (en, map) in self.entity_components.iter_mut() {
                    #nested_conditions_mut
                }

                Some(out)
            }
        };

        quote! {
            #immutable_method
            #mutable_method
        }
    });

    // Combine all generated methods into one TokenStream
    let expanded = quote! {
        #( #methods )*
    };

    TokenStream::from(expanded)
}
