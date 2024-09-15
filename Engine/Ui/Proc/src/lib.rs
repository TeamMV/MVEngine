use mvutils::enum_val_ref;
use proc_macro::TokenStream;
use quote::quote;
use xmlparser::{Token, Tokenizer};

#[proc_macro]
pub fn ui(input: TokenStream) -> TokenStream {
    let s = input.to_string();

    todo!()
}

fn parse_element(tkn: Token, stream: Tokenizer) {

}