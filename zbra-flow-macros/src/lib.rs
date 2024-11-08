use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, FnArg, Ident, ItemFn, ItemStruct, Pat, PatType, Path, Type, TypeReference
};

#[proc_macro_attribute]
pub fn flow_context(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let struct_name = &input.ident;

    let mut classes: Vec<Path> = Vec::new();

    let factory_parser = syn::meta::parser(|meta| {
        classes.push(meta.path);
        Ok(())
    });

    parse_macro_input!(attr with factory_parser);

    // eprintln!("Factory macro : {:?}", classes);

    let mut match_arms = Vec::new();
    for class in &classes {
        let class_name = class.get_ident().unwrap();
        match_arms.push(quote! {
            stringify!(#class_name) => {
                let saved_node:#class_name = serde_json::from_value(json_value).unwrap();
                ns.save_entity(saved_node)
            },
        });
    }

    let mut entities_entries = Vec::new();
    for class in &classes {
        let class_name = class.get_ident().unwrap();
        let class_name_sk = camel_to_snake_uppercase(&class_name.to_string());
        let class_name_sk = Ident::new(&class_name_sk, Span::call_site());

        entities_entries.push(quote! {
            pub const #class_name_sk: EntitySchema<#class_name> = EntitySchema{ name: stringify!(#class_name), _marker: PhantomData};
        });
    }

    let mut save_match_arms = Vec::new();
    for class in &classes {
        let class_name = class.get_ident().unwrap();
        save_match_arms.push(quote! {
            stringify!(#class_name) => {
                let nodes:Vec<#class_name> = data.iter().map(|d| serde_json::from_value(d.clone()).unwrap()).collect();
                ns.save_entities(&nodes)
            },
        });
    }

    let mut get_entities_match_arms = Vec::new();
    for class in &classes {
        let class_name = class.get_ident().unwrap();
        let class_name_sk = camel_to_snake_uppercase(&class_name.to_string());
        let class_name_sk = Ident::new(&class_name_sk, Span::call_site());

        get_entities_match_arms.push(quote! {
            stringify!(#class_name) => {
                let entities = ns.get_entities_of_kind(#struct_name::#class_name_sk, ids);
                serde_json::to_value(&entities).unwrap()
            },
        });
    }

    let mut query_property_match_arms = Vec::new();
    for class in &classes {
        let class_name = class.get_ident().unwrap();
        let class_name_sk = camel_to_snake_uppercase(&class_name.to_string());
        let class_name_sk = Ident::new(&class_name_sk, Span::call_site());

        query_property_match_arms.push(quote! {
            stringify!(#class_name) => {
                let entities = ns.query_property(#struct_name::#class_name_sk, name, expr);
                serde_json::to_value(&entities).unwrap()
            },
        });
    }

    let mut delete_match_arms = Vec::new();
    for class in &classes {
        let class_name = class.get_ident().unwrap();
        let class_name_sk = camel_to_snake_uppercase(&class_name.to_string());
        let class_name_sk = Ident::new(&class_name_sk, Span::call_site());

        delete_match_arms.push(quote! {
            stringify!(#class_name) => {
                ns.delete_entities(#struct_name::#class_name_sk, ids);
            },
        });
    }

    let mut signal_match_arms = Vec::new();
    for class in &classes {
        let class_name = class.get_ident().unwrap();
        signal_match_arms.push(quote! {
            stringify!(#class_name) => {
                let saved_node:#class_name = serde_json::from_value(data).unwrap();
                ns.execute(saved_node)
            },
        });
    }

    let expanded = quote! {

        #input

        impl #struct_name {
            #(#entities_entries)*
        }

    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn entity_hook(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    let fn_vis = &input.vis;
    let fn_block = &input.block;
    let fn_args = &input.sig.inputs;

    let first_param_sig = get_param_signature(input.sig.inputs.first());
    if first_param_sig.is_none() {
        return TokenStream::from(quote! {
            compile_error!("Function has no parameters");
        });
    }
    let (first_param_name, first_param_type) = first_param_sig.unwrap();

    let expanded = if fn_args.iter().nth(1).is_some() {
        // Extract the type of the second parameter
        let (second_param_name, second_param_type) =
            if let Some(FnArg::Typed(PatType { pat, ty, .. })) = fn_args.iter().nth(1) {
                if let Pat::Ident(ident) = &**pat {
                    let raw_ty = match **ty {
                        Type::Reference(TypeReference { ref elem, .. }) => elem,
                        _ => ty,
                    };

                    (&ident.ident, raw_ty)
                } else {
                    return TokenStream::from(quote! {
                        compile_error!("Expected the second parameter to be an identifier.");
                    });
                }
            } else {
                return TokenStream::from(quote! {
                    compile_error!("Expected a second parameter in the function signature.");
                });
            };

        let expanded = quote! {
            #fn_vis async fn #fn_name(#first_param_name: &#first_param_type, payload: Box<(dyn std::any::Any + Send + Sync)>){

                if let Some(#second_param_name) = payload.downcast_ref::<#second_param_type>() {
                    #fn_block
                } else {
                    println!("{}, Downcast error to {} ", stringify!(#fn_name), std::any::type_name::<#second_param_type>());
                }
            }
        };
        expanded
    } else {
        let expanded = quote! {

            #fn_vis fn #fn_name(ns: #first_param_type, payload: Box<&dyn std::any::Any>) {
                #fn_block
            }
        };
        expanded
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn entity_update(attr: TokenStream, item: TokenStream) -> TokenStream {
    entity_handler(attr, item, "Update")
}

#[proc_macro_attribute]
pub fn entity_delete(attr: TokenStream, item: TokenStream) -> TokenStream {
    entity_handler(attr, item, "Delete")
}

fn entity_handler(attr: TokenStream, item: TokenStream, action: &str) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;

    let mut metas = vec![];
    let metas_parser = syn::meta::parser(|meta| {
        metas.push(meta.path.clone());
        Ok(())
    });

    parse_macro_input!(attr with metas_parser);

    let action = Ident::new(action, Span::call_site());

    let entity_kind = &metas.first().unwrap();

    let entity_kind_str = entity_kind.to_token_stream().to_string();

    let fn_vis = &input.vis;
    let fn_block = &input.block;
    let fn_args = &input.sig.inputs;

    let first_param_sig = get_param_signature(input.sig.inputs.first());
    if first_param_sig.is_none() {
        return TokenStream::from(quote! {
            compile_error!("Function has no parameters");
        });
    }
    let (first_param_name, first_param_type) = first_param_sig.unwrap();

    let second_param_sig = get_param_signature(input.sig.inputs.get(1));
    if second_param_sig.is_none() {
        return TokenStream::from(quote! {
            compile_error!("Function has no parameters");
        });
    }
    let (second_param_name, second_param_type) = second_param_sig.unwrap();

    let wrapper_name = format!("{}_wrapper", fn_name);
    let wrapper_fn_name = Ident::new(&wrapper_name, Span::call_site());

    let cc_fn_name = snake_to_camel(&fn_name.to_string());
    let handler_name_str = format!("{}Handler", cc_fn_name);
    let handler_name = Ident::new(&handler_name_str, Span::call_site());

    let expanded = quote! {

        #input

        async fn #wrapper_fn_name(#first_param_name: Arc<#first_param_type>, #second_param_name: Arc<#second_param_type>) {
            #fn_name(&#first_param_name, &#second_param_name).await;
        }

        pub struct #handler_name;

        impl DataHookHandler for #handler_name {

            fn handle(&self, context: Arc<#first_param_type>, value: Arc<Payload>) -> BoxFuture<'static, ()> {
                if let Ok(data) = value.downcast::<Vec<#entity_kind>>() {
                    let future = #wrapper_fn_name(context, data);
                    Box::pin(future)
                }else {
                    Box::pin(noop())
                }
            }

            fn get_action(&self) -> EntityAction {
                EntityAction::#action
            }

            fn get_entity_kind(&self) -> &str {
                #entity_kind_str
            }

        }

    };
    TokenStream::from(expanded)
}


#[proc_macro]
pub fn entity_hooks(input: TokenStream) -> TokenStream {
    
    let mut hook_names = vec![];
    let metas_parser = syn::meta::parser(|meta| {
        hook_names.push(meta.path.clone());
        Ok(())
    });

    parse_macro_input!(input with metas_parser);

    let mut camel_case_hooks = Vec::new();
    for hook_name in &hook_names {
        let camel_case_name = snake_to_camel(&hook_name.to_token_stream().to_string());
        let handler_name = format!("{}Handler", camel_case_name);
        let ident = Ident::new(&handler_name, Span::call_site());
        camel_case_hooks.push(quote! {
            hooks.push(Box::new(#ident));
        });
    }

    let expanded = quote! {
        {
            let mut hooks: Vec<Box<dyn DataHookHandler>> = Vec::new();
            #(#camel_case_hooks)*
            hooks
        }
    };

    TokenStream::from(expanded)
}


fn get_param_signature(param: Option<&FnArg>) -> Option<(Ident, Box<Type>)> {
    if let Some(param) = param {
        match param {
            syn::FnArg::Typed(PatType { pat, ty, .. }) => {
                if let Pat::Ident(ident) = &**pat {
                    let raw_ty = match **ty {
                        Type::Reference(TypeReference { ref elem, .. }) => elem,
                        _ => ty,
                    };

                    return Some((ident.ident.clone(), raw_ty.clone()));
                }
                None
            }
            _ => None,
        }
    } else {
        None
    }
}

fn camel_to_snake_uppercase(camel: &str) -> String {
    let mut snake = String::new();
    for (i, c) in camel.chars().enumerate() {
        if c.is_uppercase() {
            if i != 0 {
                snake.push('_');
            }
            snake.push(c);
        } else {
            snake.push(c.to_ascii_uppercase());
        }
    }
    snake
}


fn snake_to_camel(snake_str: &str) -> String {
    let words = snake_str.split('_');
    let mut camel_case = String::new();
    // Capitalize the first letter of each subsequent word
    for word in words {
        let mut chars = word.chars();
        if let Some(first_char) = chars.next() {
            camel_case.push(first_char.to_ascii_uppercase());
            camel_case.extend(chars);
        }
    }

    camel_case
}
