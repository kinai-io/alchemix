use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, FnArg, Ident, ItemFn, ItemStruct, Pat, PatType, Path, Type,
    TypeReference,
};

#[proc_macro_attribute]
pub fn flux_context(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let struct_name = &input.ident;

    let mut classes: Vec<Path> = Vec::new();

    let mut hooks: Vec<Path> = Vec::new();

    let classes_parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("events") {
            let _ = meta.parse_nested_meta(|meta| {
                // let name = meta.path.get_ident().unwrap().to_string();
                classes.push(meta.path);
                Ok(())
            });
        } else if meta.path.is_ident("hooks") {
            let _ = meta.parse_nested_meta(|meta| {
                // let name = meta.path.get_ident().unwrap().to_string();
                hooks.push(meta.path);
                Ok(())
            });
        }
        // classes.push(meta.path);
        Ok(())
    });

    parse_macro_input!(attr with classes_parser);

    let mut hooks_vec_items = Vec::new();
    for hook_name in &hooks {
        hooks_vec_items.push(quote! {
            hooks.push(#hook_name());
        });
    }

    // eprintln!("Factory macro : {:?}", classes);

    let mut entities_entries = Vec::new();

    for class in &classes {
        let class_name = class.get_ident().unwrap();
        let class_name_sk = camel_to_snake_uppercase(&class_name.to_string());
        let class_name_sk = Ident::new(&class_name_sk, Span::call_site());

        entities_entries.push(quote! {
            pub const #class_name_sk: EntitySchema<#class_name> = EntitySchema{ name: stringify!(#class_name), _marker: PhantomData};
        });
    }

    let event_arms = build_event_arms(&struct_name, &classes);

    let expanded = quote! {

        #input

        impl #struct_name {
            #(#entities_entries)*
        }

        #[async_trait]
        impl FluxContext for #struct_name {

            fn as_any(&self) -> &dyn Any {
                self
            }

            fn as_context(&self) -> &dyn FluxContext {
                self
            }

            async fn json_event(&self, dispatcher: &Flux, event: &Value) -> Vec<HookResponse> {
                if let Some(kind) = event.get("kind") {
                    let kind = kind.as_str().unwrap().to_string();
                    match(kind.as_str()) {
                        #event_arms
                        _ => {
                            println!("Unknown kind {}", kind);
                            vec![]
                        },
                    }
                }else {
                    vec![]
                }
            }

            fn get_hooks(&self) -> Vec<EventHandler> {
                let mut hooks: Vec<EventHandler> = Vec::new();
                #(#hooks_vec_items)*
                hooks
            }

        }

    };

    TokenStream::from(expanded)
}

fn build_event_arms(_struct_name: &Ident, classes: &Vec<Path>) -> proc_macro2::TokenStream {
    let mut match_arms = Vec::new();
    for class in classes {
        let class_name = class.get_ident().unwrap();
        match_arms.push(quote! {
            stringify!(#class_name) => {
                if let Ok(action) = serde_json::from_value::<#class_name>(event.clone()) {
                    dispatcher.push(action).await
                }else {
                    vec![]
                }
            },
        });
    }
    let expanded = quote! {#(#match_arms)*};
    expanded
}

#[proc_macro_attribute]
pub fn flux_hook(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    let fn_inputs = &input.sig.inputs;
    let fn_output = &input.sig.output;
    let fn_body = &input.block;

    let fn_name_str = fn_name.to_string();

    let cc_fn_name = snake_to_camel(&fn_name_str);

    let executor_fn_name_str = format!("{}_executor", fn_name);
    let executor_name = Ident::new(&executor_fn_name_str, Span::call_site());

    let hook_fn_name_str = format!("{}_hook", fn_name);
    let hook_name = Ident::new(&hook_fn_name_str, Span::call_site());

    let value_param_sig = get_param_signature(input.sig.inputs.get(0));
    if value_param_sig.is_none() {
        return TokenStream::from(quote! {
            compile_error!("Function has no parameters");
        });
    }
    let (_, value_param_type) = value_param_sig.unwrap();

    let context_param_sig = get_param_signature(input.sig.inputs.get(2));
    if context_param_sig.is_none() {
        return TokenStream::from(quote! {
            compile_error!("Function has no parameters");
        });
    }
    let (_, context_param_type) = context_param_sig.unwrap();

    let trigger_kind_str = value_param_type.to_token_stream().to_string();

    let expanded = quote! {

        pub fn #fn_name() -> EventHandler {
            let handler: Pin<Box<HandlerFunction>> = Box::pin(#executor_name);
            EventHandler::new(handler, #trigger_kind_str , #cc_fn_name)
        }

        pub fn #executor_name(
            dispatcher: &Flux,
            value: Arc<Payload>,
        ) -> Pin<Box<dyn Future<Output = HookResponse> + Send + Sync + '_>> {
            let context: &#context_param_type = dispatcher.get_context();
            let state: &FluxState = dispatcher.get_state();

            Box::pin(async move {
                // Simulate some work and return an RxResponse
                if let Ok(payload) = value.downcast::<#value_param_type>() {
                    let p = payload.as_ref();
                    let mut res = #hook_name(p, state, context).await;
                    res.set_handler(#fn_name_str);
                    res
                }else {
                    let mut res = HookResponse::error( "Action downcast error");
                    res.set_handler(#fn_name_str);
                    res
                }
            })
        }

        async fn #hook_name (#fn_inputs) #fn_output
        #fn_body
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
