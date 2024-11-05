use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Field, Fields, Ident, ItemStruct};

#[proc_macro_attribute]
pub fn entity(attr: TokenStream, item: TokenStream) -> TokenStream {
    // let args = parse_macro_input!(args as MetaList);
    let input = parse_macro_input!(item as ItemStruct);
    let struct_name = &input.ident;

    // let attri: Attribute = parse_macro_input!(attr as );
    let vis = input.vis;

    let struct_name_str = struct_name.to_string();

    let mut indexed_field_name = vec![];

    let attr_parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("index") {
            meta.parse_nested_meta(|meta| {
                let name = meta.path.get_ident().unwrap().to_string();
                indexed_field_name.push(name);
                Ok(())
            })
        } else {
            Err(meta.error("unsupported factory property"))
        }
    });

    parse_macro_input!(attr with attr_parser);

    // eprintln!("Indexed fields : {:?}", &indexed_field_name);

    let mut fields = match input.fields {
        Fields::Named(ref fields) => fields.named.clone(),
        _ => panic!("Expected a struct with named fields"),
    };

    let user_fields = match input.fields {
        Fields::Named(ref fields) => fields.named.clone(),
        _ => panic!("Expected a struct with named fields"),
    };

    let user_field_names: Vec<&Ident> = user_fields
        .iter()
        .map(|f| f.ident.as_ref().unwrap())
        .collect();
    let user_field_types: Vec<&syn::Type> = user_fields.iter().map(|f| &f.ty).collect();

    let id_field: Field = syn::parse_quote! {
        pub id: String
    };
    fields.push(id_field);

    let kind_field: Field = syn::parse_quote! {
        pub kind: String
    };
    fields.push(kind_field);

    // let user_fields_with_id: Vec<_> = fields
    //     .iter()
    //     .filter(|field| {
    //         let field_name = field.ident.as_ref().unwrap().to_string();
    //         field_name != "kind"
    //     })
    //     .collect();

    // let user_field_names_with_id: Vec<&Ident> = user_fields_with_id
    //     .iter()
    //     .map(|f| f.ident.as_ref().unwrap())
    //     .collect();
    // let user_field_types_with_id: Vec<&syn::Type> =
    //     user_fields_with_id.iter().map(|f| &f.ty).collect();

    let index_fields = match input.fields {
        Fields::Named(ref fields) => fields.named.clone(),
        _ => panic!("Expected a struct with named fields"),
    };

    let index_fields = indexed_field_name
        .iter()
        .map(|name| {
            let field = index_fields.iter().find(|f| {
                if let Some(ident) = &f.ident {
                    let field_name = ident.to_string();
                    &field_name == name
                } else {
                    false
                }
            });

            if let Some(f) = field {
                let field_name = &f.ident;
                let field_type = &f.ty;
                let type_marker = match field_type {
                    syn::Type::Path(type_path) => {
                        let segment = &type_path.path.segments.last().unwrap().ident;
                        if segment == "String" {
                            Some(quote! {
                                FieldIndex {
                                    kind: stringify!(#struct_name).to_string(),
                                    entity_id: self.id.to_string(),
                                    name: stringify!(#field_name).to_string(),
                                    value: self.#field_name.to_string(),
                                    stored_type: "String".to_string()
                                }
                            })
                        } else if segment == "usize" || segment == "u16" || segment == "u32" {
                            Some(quote! {
                                FieldIndex {
                                    kind: stringify!(#struct_name).to_string(),
                                    entity_id: self.id.to_string(),
                                    name: stringify!(#field_name).to_string(),
                                    value: self.#field_name.to_string(),
                                    stored_type: "String".to_string()
                                }
                            })
                        } else {
                            None
                        }
                    }
                    _ => None,
                };
                type_marker
            } else {
                panic!("Unknown field to index")
            }
        })
        .collect::<Vec<_>>();

    let expanded = quote! {
        #[derive(Debug, Serialize, Deserialize, Clone, TS)]
        #[ts(export)]
        #vis struct #struct_name {
            #fields
        }

        impl #struct_name {

            pub fn new(#(#user_field_names: #user_field_types),*) -> Self {
                Self {
                    #(#user_field_names),*,  // Populate user-defined fields
                    kind: stringify!(#struct_name).to_string(),  // Automatically set kind to struct name
                    id: Uuid::new_v4().to_string(),  // Generate a unique ID
                }
            }

            pub fn new_with_id(id: &str, #(#user_field_names: #user_field_types),*) -> Self {
                Self {
                    #(#user_field_names),*,  // Populate user-defined fields
                    kind: stringify!(#struct_name).to_string(),  // Automatically set kind to struct name
                    id: id.to_string(),
                }
            }

        }

        impl Entity for #struct_name {

            fn get_id(&self) -> &str {
                &self.id
            }

            fn get_kind(&self) -> &str {
                #struct_name_str
            }

            fn get_key(&self) -> String{
                format!("{}#{}", self.get_kind(), &self.id)
            }

            fn get_fields_index(&self) -> Vec<FieldIndex> {
                vec![#(#index_fields),*]
            }

        }

    };

    TokenStream::from(expanded)
}
