//! Derive macro for the Reflect trait.
//!
//! This crate provides the `#[derive(Reflect)]` macro for automatic
//! implementation of the Reflect trait on structs, enums, and tuples.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Type};

/// Derive macro for the Reflect trait.
///
/// # Examples
///
/// ```ignore
/// use luminara_core::Reflect;
///
/// #[derive(Reflect)]
/// struct Transform {
///     position: Vec3,
///     rotation: Quat,
///     scale: Vec3,
/// }
/// ```
#[proc_macro_derive(Reflect)]
pub fn derive_reflect(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let type_name = name.to_string();
    let full_type_name = format!("{}::{}", std::env::var("CARGO_PKG_NAME").unwrap_or_default(), type_name);

    let expanded = match &input.data {
        Data::Struct(data_struct) => {
            impl_reflect_struct(name, &impl_generics, &ty_generics, where_clause, &full_type_name, &data_struct.fields)
        }
        Data::Enum(data_enum) => {
            impl_reflect_enum(name, &impl_generics, &ty_generics, where_clause, &full_type_name, data_enum)
        }
        Data::Union(_) => {
            panic!("Reflect cannot be derived for unions");
        }
    };

    TokenStream::from(expanded)
}

fn impl_reflect_struct(
    name: &syn::Ident,
    impl_generics: &syn::ImplGenerics,
    ty_generics: &syn::TypeGenerics,
    where_clause: Option<&syn::WhereClause>,
    full_type_name: &str,
    fields: &Fields,
) -> proc_macro2::TokenStream {
    let (field_info_init, field_match, field_mut_match, set_field_match, serialize_fields, deserialize_fields) = match fields {
        Fields::Named(fields_named) => {
            let field_names: Vec<_> = fields_named.named.iter().map(|f| &f.ident).collect();
            let field_types: Vec<_> = fields_named.named.iter().map(|f| &f.ty).collect();
            let field_name_strs: Vec<_> = field_names.iter().map(|n| n.as_ref().unwrap().to_string()).collect();

            let field_info = quote! {
                vec![
                    #(
                        luminara_core::FieldInfo {
                            name: #field_name_strs.to_string(),
                            type_name: std::any::type_name::<#field_types>().to_string(),
                            type_id: std::any::TypeId::of::<#field_types>(),
                            description: None,
                            default_value: None,
                        }
                    ),*
                ]
            };

            let field_match = quote! {
                match name {
                    #(
                        #field_name_strs => Some(&self.#field_names as &dyn luminara_core::Reflect),
                    )*
                    _ => None,
                }
            };

            let field_mut_match = quote! {
                match name {
                    #(
                        #field_name_strs => Some(&mut self.#field_names as &mut dyn luminara_core::Reflect),
                    )*
                    _ => None,
                }
            };

            let set_field_match = quote! {
                match name {
                    #(
                        #field_name_strs => {
                            if let Some(concrete) = value.as_any().downcast_ref::<#field_types>() {
                                self.#field_names = concrete.clone();
                                Ok(())
                            } else {
                                Err(luminara_core::ReflectError::TypeMismatch {
                                    expected: std::any::type_name::<#field_types>().to_string(),
                                    actual: value.type_info().type_name.clone(),
                                })
                            }
                        }
                    )*
                    _ => Err(luminara_core::ReflectError::FieldNotFound(name.to_string())),
                }
            };

            let serialize_fields = quote! {
                let mut map = serde_json::Map::new();
                #(
                    map.insert(#field_name_strs.to_string(), self.#field_names.serialize_json());
                )*
                serde_json::Value::Object(map)
            };

            let deserialize_fields = quote! {
                if let serde_json::Value::Object(map) = value {
                    #(
                        if let Some(field_value) = map.get(#field_name_strs) {
                            self.#field_names.deserialize_json(field_value)?;
                        }
                    )*
                    Ok(())
                } else {
                    Err(luminara_core::ReflectError::DeserializationError(
                        "Expected object".to_string()
                    ))
                }
            };

            (field_info, field_match, field_mut_match, set_field_match, serialize_fields, deserialize_fields)
        }
        Fields::Unnamed(fields_unnamed) => {
            let field_count = fields_unnamed.unnamed.len();
            let field_indices: Vec<syn::Index> = (0..field_count).map(|i| syn::Index::from(i)).collect();
            let field_types: Vec<_> = fields_unnamed.unnamed.iter().map(|f| &f.ty).collect();
            let field_index_strs: Vec<_> = (0..field_count).map(|i| i.to_string()).collect();

            let field_info = quote! {
                vec![
                    #(
                        luminara_core::FieldInfo {
                            name: #field_index_strs.to_string(),
                            type_name: std::any::type_name::<#field_types>().to_string(),
                            type_id: std::any::TypeId::of::<#field_types>(),
                            description: None,
                            default_value: None,
                        }
                    ),*
                ]
            };

            let field_match = quote! {
                match name {
                    #(
                        #field_index_strs => Some(&self.#field_indices as &dyn luminara_core::Reflect),
                    )*
                    _ => None,
                }
            };

            let field_mut_match = quote! {
                match name {
                    #(
                        #field_index_strs => Some(&mut self.#field_indices as &mut dyn luminara_core::Reflect),
                    )*
                    _ => None,
                }
            };

            let set_field_match = quote! {
                match name {
                    #(
                        #field_index_strs => {
                            if let Some(concrete) = value.as_any().downcast_ref::<#field_types>() {
                                self.#field_indices = concrete.clone();
                                Ok(())
                            } else {
                                Err(luminara_core::ReflectError::TypeMismatch {
                                    expected: std::any::type_name::<#field_types>().to_string(),
                                    actual: value.type_info().type_name.clone(),
                                })
                            }
                        }
                    )*
                    _ => Err(luminara_core::ReflectError::FieldNotFound(name.to_string())),
                }
            };

            let serialize_fields = quote! {
                let array = vec![
                    #(
                        self.#field_indices.serialize_json(),
                    )*
                ];
                serde_json::Value::Array(array)
            };

            let field_indices_for_get: Vec<usize> = (0..field_count).collect();
            let deserialize_fields = quote! {
                if let serde_json::Value::Array(array) = value {
                    #(
                        if let Some(field_value) = array.get(#field_indices_for_get) {
                            self.#field_indices.deserialize_json(field_value)?;
                        }
                    )*
                    Ok(())
                } else {
                    Err(luminara_core::ReflectError::DeserializationError(
                        "Expected array".to_string()
                    ))
                }
            };

            (field_info, field_match, field_mut_match, set_field_match, serialize_fields, deserialize_fields)
        }
        Fields::Unit => {
            let field_info = quote! { vec![] };
            let field_match = quote! { None };
            let field_mut_match = quote! { None };
            let set_field_match = quote! {
                Err(luminara_core::ReflectError::FieldNotFound(name.to_string()))
            };
            let serialize_fields = quote! { serde_json::Value::Null };
            let deserialize_fields = quote! { Ok(()) };

            (field_info, field_match, field_mut_match, set_field_match, serialize_fields, deserialize_fields)
        }
    };

    let type_kind = match fields {
        Fields::Named(_) => quote! { luminara_core::TypeKind::Struct },
        Fields::Unnamed(_) => quote! { luminara_core::TypeKind::Tuple },
        Fields::Unit => quote! { luminara_core::TypeKind::Value },
    };

    quote! {
        impl #impl_generics luminara_core::Reflect for #name #ty_generics #where_clause {
            fn type_info(&self) -> &luminara_core::TypeInfo {
                use std::sync::OnceLock;
                static INFO: OnceLock<luminara_core::TypeInfo> = OnceLock::new();
                INFO.get_or_init(|| luminara_core::TypeInfo {
                    type_name: #full_type_name.to_string(),
                    type_id: std::any::TypeId::of::<#name #ty_generics>(),
                    kind: #type_kind,
                    fields: #field_info_init,
                })
            }

            fn field(&self, name: &str) -> Option<&dyn luminara_core::Reflect> {
                #field_match
            }

            fn field_mut(&mut self, name: &str) -> Option<&mut dyn luminara_core::Reflect> {
                #field_mut_match
            }

            fn set_field(&mut self, name: &str, value: Box<dyn luminara_core::Reflect>) -> Result<(), luminara_core::ReflectError> {
                #set_field_match
            }

            fn clone_value(&self) -> Box<dyn luminara_core::Reflect> {
                Box::new(self.clone())
            }

            fn serialize_json(&self) -> serde_json::Value {
                #serialize_fields
            }

            fn deserialize_json(&mut self, value: &serde_json::Value) -> Result<(), luminara_core::ReflectError> {
                #deserialize_fields
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }
        }
    }
}

fn impl_reflect_enum(
    name: &syn::Ident,
    impl_generics: &syn::ImplGenerics,
    ty_generics: &syn::TypeGenerics,
    where_clause: Option<&syn::WhereClause>,
    full_type_name: &str,
    data_enum: &syn::DataEnum,
) -> proc_macro2::TokenStream {
    let variant_names: Vec<_> = data_enum.variants.iter().map(|v| &v.ident).collect();
    let variant_name_strs: Vec<_> = variant_names.iter().map(|n| n.to_string()).collect();

    // For simplicity, enums are treated as values with no field access
    // A more complete implementation would handle enum variants with fields
    quote! {
        impl #impl_generics luminara_core::Reflect for #name #ty_generics #where_clause {
            fn type_info(&self) -> &luminara_core::TypeInfo {
                use std::sync::OnceLock;
                static INFO: OnceLock<luminara_core::TypeInfo> = OnceLock::new();
                INFO.get_or_init(|| luminara_core::TypeInfo {
                    type_name: #full_type_name.to_string(),
                    type_id: std::any::TypeId::of::<#name #ty_generics>(),
                    kind: luminara_core::TypeKind::Enum,
                    fields: vec![],
                })
            }

            fn field(&self, _name: &str) -> Option<&dyn luminara_core::Reflect> {
                None
            }

            fn field_mut(&mut self, _name: &str) -> Option<&mut dyn luminara_core::Reflect> {
                None
            }

            fn set_field(&mut self, name: &str, _value: Box<dyn luminara_core::Reflect>) -> Result<(), luminara_core::ReflectError> {
                Err(luminara_core::ReflectError::FieldNotFound(name.to_string()))
            }

            fn clone_value(&self) -> Box<dyn luminara_core::Reflect> {
                Box::new(self.clone())
            }

            fn serialize_json(&self) -> serde_json::Value {
                // Basic enum serialization - just the variant name
                let variant_name = match self {
                    #(
                        #name::#variant_names { .. } => #variant_name_strs,
                    )*
                };
                serde_json::Value::String(variant_name.to_string())
            }

            fn deserialize_json(&mut self, _value: &serde_json::Value) -> Result<(), luminara_core::ReflectError> {
                Err(luminara_core::ReflectError::DeserializationError(
                    "Enum deserialization not yet implemented".to_string()
                ))
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }
        }
    }
}
