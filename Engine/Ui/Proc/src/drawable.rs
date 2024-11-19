use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use ui_parsing::xml;
use ui_parsing::xml::{Entity, XmlValue};

pub fn drawable(input: TokenStream) -> TokenStream {
    let str = input.to_string();
    let rsx = xml::parse_rsx(str).unwrap();
    let out = process_drawable(&rsx);
    TokenStream::from(out)
}

fn process_drawable(d: &Entity) -> proc_macro2::TokenStream {
    let name = d.name();
    let inner = d.inner().as_ref();

    let mut drawables_vec = quote! { vec![] };
    if let Some(inner_value) = inner {
        if let XmlValue::Entities(inner_d) = inner_value {
            let drawables: Vec<proc_macro2::TokenStream> = inner_d
                .iter()
                .map(|ent| process_drawable(ent))
                .collect();

            drawables_vec = quote! {
                vec![
                    #(#drawables),*
                ]
            };
        }
    }

    let mut attribs_ts = quote! {
        let mut __map__ = HashMap::new();
    };

    for attribute in d.attributes() {
        let name = attribute.name();
        let value = match attribute.value() {
            XmlValue::Str(s) => s.clone(),
            _ => panic!("Only strings can be used as attribute value!")
        };
        attribs_ts.extend(quote! {
            __map__.insert(#name, #value);
        });
    }

    attribs_ts.extend(quote! {__map__});

    let attribs = quote! {
        {
            #attribs_ts
        }
    };

    let ident = Ident::new(name.as_str(), Span::call_site());

    quote! {
        #ident::create(#drawables_vec, #attribs).unwrap()
    }
}