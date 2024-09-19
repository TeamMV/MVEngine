use proc_macro::TokenStream;
use std::fmt::{Debug, Formatter};
use proc_macro2::Ident;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Block, Expr, ExprCall, ExprPath, GenericArgument, ItemFn, Local, Pat, PathArguments, Stmt};

use proc_macro2 as pm2;
use syn::parse::{Parse, ParseStream};

pub fn uix(_attrib: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let vis = &input.vis;
    let comp_name = &input.sig.ident;
    let parameters = &input.sig.inputs;
    let returns = &input.sig.output;

    let block = input.block.as_ref();

    let (local_states, global_states, replaced_block) = get_state_uses(block);

    let state_mod = quote! { mvutils::state };

    let mut state_fields = quote! {};
    for local_state in local_states.iter() {
        let name = &local_state.var_name;
        let ty = &local_state.var_type;

        state_fields.extend(quote! {
            #name: #state_mod::State<#ty>
        });
    }

    let struct_code = quote! {
        #vis struct #comp_name {
            _cached: mvutils::once::CreateOnce<mvengine_ui::uix::DynamicUi>,
            attributes: mvengine_ui::attributes::Attributes,
            style: mvengine_ui::styles::UiStyle,
            #state_fields
        }
    };

    let mut state_init_code = quote! {};
    for local_state in local_states.iter() {
        let name = &local_state.var_name;
        let init = &local_state.init_tokens;
        state_init_code.extend(quote! {
            let #name = #state_mod::State::new(#init);
        });
    }
    for global_state in global_states.iter() {
        let name = &global_state.var_name;
        let init = &global_state.init_tokens;
        state_init_code.extend(quote! {
            let #name = #init;
        });
    }

    let mut field_init_code = quote! {
        attributes: attributes.unwrap_or(mvengine_ui::attributes::Attributes::new()),
        style: style.unwrap_or(mvengine_ui::styles::UiStyle::default()),
        _cached: mvutils::once::CreateOnce::new(),
    };

    for local_state in local_states.iter() {
        let name = &local_state.var_name;
        field_init_code.extend(quote! {
            #name
        });
    }
    for global_state in global_states.iter() {
        let name = &global_state.var_name;
        field_init_code.extend(quote! {
            #name
        });
    }

    let mut state_when_code = quote! {if };
    for local_state in local_states.iter() {
        let name = &local_state.var_name;
        state_when_code.extend(quote! {
            self.#name.is_outdated() ||
        });
    }
    for global_state in global_states.iter() {
        let name = &global_state.var_name;
        state_when_code.extend(quote! {
            self.#name.is_outdated() ||
        });
    }
    state_when_code.extend(quote! {
        false {
            self._cached.regenerate();
        } else {
            self._cached.check_children();
        }
    });

    let struct_impl = quote! {
        impl mvengine_ui::uix::UiCompoundElement for #comp_name {
            fn new(attributes: Option<mvengine_ui::attributes::Attributes>, style: Option<mvengine_ui::styles::UiStyle>) -> Self where Self: Sized {
                #state_init_code
                Self {
                    #field_init_code
                }
            }

            fn generate(&self) #returns {
                let _ = self._cached.try_create(|| mvengine_ui::uix::DynamicUi::new(|| self.generate()));

                #replaced_block
            }

            fn regenerate(&mut self) {
                #state_when_code
            }
        }
    };

    let q = quote! {
        #struct_code
        #struct_impl
    };
    q.into()
}

struct StateUse {
    var_name: Ident,
    var_type: pm2::TokenStream,
    init_tokens: pm2::TokenStream
}

impl Debug for StateUse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StateUse")
            .field("name", &self.var_name)
            .field("type", &self.var_type)
            .field("init", &self.init_tokens)
            .finish()
    }
}

fn get_state_uses(fn_block: &Block) -> (Vec<StateUse>, Vec<StateUse>, pm2::TokenStream) {
    let mut local_usages = Vec::new();
    let mut global_usages = Vec::new();

    let mut rep_block = quote! {};

    for stmt in fn_block.stmts.iter() {
        if let Stmt::Local(Local { pat, init, .. }) = stmt {
            if let Some(loc_init) = init {
                let expr = loc_init.expr.as_ref();

                if let Expr::Call(ExprCall { func, args, .. }) = expr {
                    if let Expr::Path(ExprPath { path, .. }) = func.as_ref() {
                        if path.segments.first().unwrap().ident.to_string() == "use_state".to_string() {
                            let type_ts = if let PathArguments::AngleBracketed(generic_args) = &path.segments.last().unwrap().arguments {
                                let g = generic_args.args.first().unwrap();
                                if let GenericArgument::Type(ty) = g {
                                    ty.to_token_stream()
                                } else {
                                    panic!("Generic Argument was not a type!")
                                }
                            } else {
                                panic!("use_state() needs a generic type!");
                            };


                            let init_ts = if let Some(init_expr) = args.first() {
                                init_expr.to_token_stream()
                            } else {
                                panic!("Initial value has to be supplied when using a state")
                            };

                            let var_name_ts = if let Pat::Ident(pat_ident) = pat {
                                pat_ident.ident.clone()
                            } else {
                                panic!("use_state() can only be used to assign a variable.")
                            };

                            rep_block.extend(quote! {
                                let #var_name_ts = self.#var_name_ts;
                            });

                            local_usages.push(StateUse {
                                var_name: var_name_ts,
                                var_type: type_ts,
                                init_tokens: init_ts,
                            });
                            continue;
                        }
                    }
                }
            }
        }

        rep_block.extend(quote! {
            #stmt
        });
    }

    (local_usages, global_usages, rep_block)
}

fn get_return_code(block: &Block) -> pm2::TokenStream {
    for stmt in block.stmts.iter() {
        if let Stmt::Expr(expr, ..) = stmt {
            if let Expr::Return(return_expr) = expr {
                return if let Some(return_value) = &return_expr.expr {
                    return_value.to_token_stream()
                } else {
                    quote::quote! { () }
                }
            }
        }
    }

    if let Some(Stmt::Expr(last_expr, ..)) = block.stmts.last() {
        return last_expr.to_token_stream();
    }

    panic!("Function has to return a UiValue!");
}