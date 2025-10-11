use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{parse_macro_input, ItemStruct};

pub fn cached_hash(input: TokenStream) -> TokenStream {
    let s = parse_macro_input!(input as ItemStruct);

    let vis = s.vis;
    let attrs = s.attrs;
    let ident = s.ident;
    let inner_ident = Ident::new(&format!("{}__CHASH", ident), Span::call_site());
    let fields = s.fields;

    let ts = quote! {
        #(#attrs)*
        #vis struct #ident {
            cached_hash: std::cell::Cell<Option<u64>>,
            inner: #inner_ident,
        }

        #(#attrs)*
        #[derive(std::hash::Hash)]
        #vis struct #inner_ident #fields

        impl std::ops::Deref for #ident {
            type Target = #inner_ident;
            fn deref(&self) -> &Self::Target {
                &self.inner
            }
        }

        impl std::ops::DerefMut for #ident {
            fn deref_mut(&mut self) -> &mut Self::Target {
                self.cached_hash.set(None);
                &mut self.inner
            }
        }

        impl std::hash::Hash for #ident {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                use std::hash::{Hash, Hasher};
                if let Some(cached) = self.cached_hash.get() {
                    state.write_u64(cached);
                } else {
                    let mut hasher = std::collections::hash_map::DefaultHasher::new();
                    self.inner.hash(&mut hasher);
                    let h = hasher.finish();
                    self.cached_hash.set(Some(h));
                    state.write_u64(h);
                }
            }
        }
    };

    ts.into()
}
