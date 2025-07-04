use proc_macro::TokenStream;
use std::str::FromStr;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, FnArg, ItemFn};

fn map_type(ty: &str) -> String {
    if ty.starts_with("Option <") {
        let ty = ty.strip_prefix("Option < ").unwrap().strip_suffix(" >").unwrap();
        format!("as_{}_nullable", ty.to_lowercase())
    } else {
        format!("as_{}", ty.to_lowercase())
    }
}

pub fn msfx_fn(attr: TokenStream, body: TokenStream) -> TokenStream {
    let function = parse_macro_input!(body as ItemFn);

    let attr = attr.to_string();

    let name = &function.sig.ident;
    let s_name = if attr.is_empty() {
        name.to_string().split('_').map(|w| { let mut c = w.chars(); c.next().map(|f| f.to_ascii_uppercase()).into_iter().chain(c).collect::<String>() }).collect::<String>()
    } else {
        attr
    };
    let s_name = proc_macro2::TokenStream::from_str(&s_name).unwrap();

    let mut mapping = quote! {};
    let mut args = quote! {};

    if function.sig.inputs.len() == 1 {
        let FnArg::Typed(var) = &function.sig.inputs[0] else { panic!("Cannot accept `self` to msfx function") };
        let str = var.to_token_stream().to_string();
        let Some((name, ty)) = str.split_once(':') else { unreachable!() };
        let name = name.trim();
        let ty_fn = map_type(ty.trim());

        args.extend(proc_macro2::TokenStream::from_str(&format!("{name}")));
        mapping.extend(proc_macro2::TokenStream::from_str(&format!("let {name} = get_unnamed(&arguments, \"{name}\").{ty_fn}()?;")));
    } else if !function.sig.inputs.is_empty() {
        for arg in &function.sig.inputs {
            let FnArg::Typed(var) = arg else { panic!("Cannot accept `self` to msfx function") };
            let str = var.to_token_stream().to_string();
            let Some((name, ty)) = str.split_once(':') else { unreachable!() };
            let name = name.trim();
            let ty_fn = map_type(ty.trim());

            if name == "__actual_literal_underscore_lmao" {
                args.extend(proc_macro2::TokenStream::from_str(&format!("{name}, ")));
                mapping.extend(proc_macro2::TokenStream::from_str(&format!("let {name} = get_named(&arguments, \"_\").{ty_fn}()?;")));
            } else {
                args.extend(proc_macro2::TokenStream::from_str(&format!("{name}, ")));
                mapping.extend(proc_macro2::TokenStream::from_str(&format!("let {name} = get_named(&arguments, \"{name}\").{ty_fn}()?;")));
            }
        }
    }

    let map = if function.sig.output.to_token_stream().to_string().trim().starts_with("-> Result") {
        quote!{
            Self::#name(#args).map(Into::into)
        }
    } else {
        quote! {
            Ok(Self::#name(#args).into())
        }
    };

    let ts = quote! {
        pub struct #s_name;

        impl #s_name {
            #function
        }

        impl MSFXFunction for #s_name {
            fn call(&self, arguments: HashMap<String, MappedVariable>) -> Result<MappedVariable, String> {
                #mapping
                #map
            }
        }
    };

    ts.into()
}