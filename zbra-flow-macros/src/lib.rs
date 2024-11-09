use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, FnArg, Ident, ItemFn, ItemStruct, Pat, PatType, Path, Type, TypeReference,
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

    let mut entities_entries = Vec::new();
    
    for class in &classes {
        let class_name = class.get_ident().unwrap();
        let class_name_sk = camel_to_snake_uppercase(&class_name.to_string());
        let class_name_sk = Ident::new(&class_name_sk, Span::call_site());

        entities_entries.push(quote! {
            pub const #class_name_sk: EntitySchema<#class_name> = EntitySchema{ name: stringify!(#class_name), _marker: PhantomData};
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
    let action = Ident::new(action, Span::call_site());

    let mut metas = vec![];
    let metas_parser = syn::meta::parser(|meta| {
        metas.push(meta.path.clone());
        Ok(())
    });
    parse_macro_input!(attr with metas_parser);

    let entity_kind = &metas.first().unwrap();
    let entity_kind_str = entity_kind.to_token_stream().to_string();

    let context_param_sig = get_param_signature(input.sig.inputs.get(2));
   
    // let (context_param_name, context_param_type) = context_param_sig.unwrap();

    let value_param_sig = get_param_signature(input.sig.inputs.get(0));
    if value_param_sig.is_none() {
        return TokenStream::from(quote! {
            compile_error!("Function has no parameters");
        });
    }
    let (value_param_name, value_param_type) = value_param_sig.unwrap();

    let wrapper_name = format!("{}_wrapper", fn_name);
    let wrapper_fn_name = Ident::new(&wrapper_name, Span::call_site());

    let cc_fn_name = snake_to_camel(&fn_name.to_string());
    let handler_name_str = format!("{}Handler", cc_fn_name);
    let handler_name = Ident::new(&handler_name_str, Span::call_site());

    let payload_type_name = Ident::new("DispatchPayload", Span::call_site());

    let invocation = if let Some((_, context_param_type)) = context_param_sig {
        quote! {
            if let Some(context) = payload.store.get_context::<#context_param_type>() {
                #fn_name(&#value_param_name, payload.store, context).await;
            }
        }
    }else {
        quote! {
            #fn_name(&#value_param_name, payload.store).await;
        }
    };

    let expanded = quote! {

        #input

        async fn #wrapper_fn_name(payload: Arc<#payload_type_name<'_>>, #value_param_name: Arc<#value_param_type>) {
            #invocation;
        }

        pub struct #handler_name;

        #[async_trait]
        impl DataHookHandler for #handler_name {

            async fn handle(&self, payload: Arc<DispatchPayload<'_>>, value: Arc<Payload>){
                if let Ok(data) = value.downcast::<Vec<#entity_kind>>() {
                    #wrapper_fn_name(payload, data).await;
                }else {
                    println!("Downcast Error");
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
            let mut hooks: Vec<Box<dyn DataHookHandler + Send + Sync>> = Vec::new();
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
