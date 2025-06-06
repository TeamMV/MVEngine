use proc_macro::TokenStream;
use std::ops::Deref;
use quote::quote;
use syn::{parse_macro_input, FnArg, ImplItem, ItemImpl, Path, Type};

pub fn listener(head: TokenStream, body: TokenStream) -> TokenStream {
    let event_enum_path = parse_macro_input!(head as Path);
    let mut enum_match_ts = quote! {};

    let impl_block = parse_macro_input!(body as ItemImpl);

    let struct_name = if let Type::Path(path) = impl_block.self_ty.deref() {
        &path.path.segments.last().unwrap().ident
    } else {
        panic!("Expected impl target to be a simple path (like `impl MyStruct`)!");
    };

    let mut methods_ts = quote! {};

    for method in impl_block.items.iter().filter_map(|item| {
        if let ImplItem::Fn(method) = item {
            Some(method)
        } else {
            None
        }
    }) {
        let mut has_self = false;
        let mut found_event_ty = false;
        let mut event_type = None;
        let mut uses_queue = false;
        for arg in &method.sig.inputs {
            match arg {
                FnArg::Receiver(_) => { has_self = true; }
                FnArg::Typed(arg) => {
                    if found_event_ty {
                        //must be queue argument
                        if !uses_queue {
                            uses_queue = true;
                        } else {
                            panic!("An event receiver may only take the event itself and an optional &mut EventQueue<Event>!");
                        }
                    } else {
                        match arg.ty.deref() {
                            Type::Reference(reference) => {
                                if reference.mutability.is_some() {
                                    if let Type::Path(path) = reference.elem.deref() {
                                        event_type = Some(path.clone());
                                        found_event_ty = true;
                                    }
                                } else {
                                    panic!("The event argument has to be mutable, e.g. '&mut PlayerJoinEvent'");
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        if !has_self {
            panic!("An event receiver fn has to take in a 'self'!");
        }

        if let Some(path) = event_type {
            if let Some(last) = path.path.segments.last() {
                let ident = &last.ident;
                let method_ident = &method.sig.ident;
                if !uses_queue {
                    enum_match_ts.extend(quote! {
                        #event_enum_path::#ident(a) => self.#method_ident(a),
                    });
                } else {
                    enum_match_ts.extend(quote! {
                        #event_enum_path::#ident(a) => self.#method_ident(a, queue),
                    });
                }
            }
        }
        methods_ts.extend(quote! { #method });
    }

    let ts = quote! {
        impl #struct_name {
            #methods_ts
        }

        impl mvengine::event::EventReceiver<#event_enum_path> for #struct_name {
            fn on_dispatch(&mut self, event: &mut #event_enum_path, queue: &mut mvengine::event::EventQueue<#event_enum_path>) {
                match event {
                    #enum_match_ts
                    _ => {}
                }
            }
        }
    };

    ts.into()
}