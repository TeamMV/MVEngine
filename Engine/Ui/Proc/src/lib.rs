use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens};
use syn::{parse_str, Expr};
use ui_parsing::xml::{Entity, XmlValue};

#[proc_macro]
pub fn ui(input: TokenStream) -> TokenStream {
    let s = input.to_string();
    let mut chars = s.chars();
    let mut chars = chars.skip_while(|c| c.is_whitespace());
    if !chars.next().is_some_and(|c| c == '[') { panic!("Expected form '[states] => {{rsx}}]'") }
    let mut dep_str = String::new();
    loop {
        let c = chars.next().unwrap_or_else(|| { panic!("Expected form '[states] => {{rsx}}]'") });
        if c == ']' {
            break;
        }
        dep_str.push(c);
    }

    let mut chars = chars.skip_while(|c| c.is_whitespace());
    if !chars.next().is_some_and(|c| c == '=') { panic!("Expected form '[states] => {{rsx}}]'") }
    if !chars.next().is_some_and(|c| c == '>') { panic!("Expected form '[states] => {{rsx}}]'") }
    let mut chars = chars.skip_while(|c| c.is_whitespace());
    if !chars.next().is_some_and(|c| c == '{') { panic!("Expected form '[states] => {{rsx}}]'") }

    let rsx_raw = chars.collect::<String>();
    let rsx_raw = rsx_raw.strip_suffix("}").unwrap_or_else(|| { panic!("Expected form '[states] => {{rsx}}]'") }).to_string();

    let rsx = ui_parsing::xml::parse_rsx(rsx_raw).unwrap_or_else(|err| panic!("{}", err));
    let tree = parse_entity(&rsx);

    let when_str = format!("mvutils::state::when!([{}] => {{this.cached = {}}})", dep_str, tree.to_string());
    let parsed_when: Expr = parse_str(&when_str).expect("Failed to parse code as expression");

    let q = quote! {
        mvengine_ui::create::DynamicUi::new(#tree, |this| {#parsed_when})
    };
    q.into()
}

fn parse_entity(entity: &Entity) -> proc_macro2::TokenStream {
    let new_ui_style: XmlValue = XmlValue::Code("mvengine_ui::styles::UiStyle::default()".to_string());
    let new_attributes: XmlValue = XmlValue::Code("mvengine_ui::attributes::Attributes::new()".to_string());

    let id = mvutils::utils::next_id("MVEngine::ui::proc_parse_entity").to_string();
    let attribs_ident = Ident::new(&format!("__attributes_{}__", id), Span::call_site());

    let name = entity.name();
    let style_xml = entity.get_attrib("style").unwrap_or(&new_ui_style);
    let style_code = xml_value_to_tknstream(style_xml);

    let attributes_xml = entity.get_attrib("attributes").unwrap_or(&new_attributes);
    let attributes_code = xml_value_to_tknstream(attributes_xml);

    let mut attrib_tokens = quote! {};
    for attrib in entity.attributes().iter().filter(|a| a.name() != "style".to_string() && a.name() != "attributes".to_string()) {
        let attrib_name = attrib.name();
        let attrib_value_xml = attrib.value();
        let attrib_value = match attrib_value_xml {
            XmlValue::Str(s) => {
                quote! {
                    mvengine_ui::attributes::AttributeValue::Str(#s.to_string())
                }
            },
            XmlValue::Entities(_) => unreachable!(),
            XmlValue::Code(c) => {
                let parsed_code: Expr = parse_str(&c).expect("Failed to parse code as expression");
                quote! {
                    mvengine_ui::attributes::AttributeValue::Code(Box::new(#parsed_code))
                }
            },
        };

        attrib_tokens.extend(quote! {
            #attribs_ident = #attribs_ident.with_attrib(#attrib_name.to_string(), #attrib_value);
        });
    }

    let elem_ident = Ident::new(&format!("__{}_{}__", name.to_lowercase(), id), Span::call_site());
    let name_ident = Ident::new(&name, Span::call_site());

    let inner = entity.inner().as_ref();
    let inner_code = if inner.is_some() {
        let inner_xml = inner.unwrap();
        match inner_xml {
            XmlValue::Str(s) => {
                quote! {
                    #attribs_ident = #attribs_ident.with_inner(mvengine_ui::attributes::AttributeValue::Str(#s.to_string()));
                    #elem_ident.state_mut().children.push(mvengine_ui::elements::child::Child::String(#s.to_string()))
                }
            }
            XmlValue::Entities(e) => {
                let mut en_qt = quote! {};
                for en in e {
                    let ts = parse_entity(en);
                    en_qt.extend(quote! {
                        #elem_ident.state_mut().children.push(mvengine_ui::elements::child::Child::Element(std::sync::Arc::new(parking_lot::RwLock::new(#ts))));
                    });
                }
                en_qt
            }
            XmlValue::Code(c) => {
                let parsed_code: Expr = parse_str(&c).expect("Failed to parse code as expression");
                quote! {
                    #attribs_ident = #attribs_ident.with_inner(mvengine_ui::attributes::AttributeValue::Code(Box::new(#parsed_code)))
                }
            }
        }
    } else {
        quote! {}
    };

    let q = quote! {
        {
            let mut #attribs_ident = #attributes_code;
            #attrib_tokens

            let mut #elem_ident = #name_ident::new(#attribs_ident, #style_code);
            #inner_code;
            #elem_ident.wrap()
        }
    };
    q
}

fn xml_value_to_tknstream(value: &XmlValue) -> proc_macro2::TokenStream {
    match value {
        XmlValue::Str(s) => { todo!() }
        XmlValue::Code(c) => {
            let parsed_code: Expr = parse_str(&c).expect("Failed to parse code as expression");
            quote! { #parsed_code }
        },
        _ => { unreachable!() }
    }
}