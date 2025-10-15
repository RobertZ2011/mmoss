use quote::quote;
use syn::{Data, DeriveInput, Expr, Meta, parse_macro_input};

#[proc_macro_derive(Replicated, attributes(replicated, replication_id))]
pub fn derive_replicated(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut replication_id_field = None;
    let mut serialize = Vec::new();
    let mut deserialize = Vec::new();

    match input.data {
        Data::Struct(data) => {
            for field in data.fields.iter() {
                for attr in &field.attrs {
                    if attr.path().is_ident("replication_id") {
                        if replication_id_field.is_some() {
                            return syn::Error::new_spanned(
                                field,
                                "Only one field can be marked with replication_id",
                            )
                            .to_compile_error()
                            .into();
                        }
                        replication_id_field = Some(field.ident.clone());
                    }

                    if attr.path().is_ident("replicated") {
                        let mut into_from_path = None;

                        if let Meta::List(_) = attr.meta {
                            let result = attr.parse_nested_meta(|meta| {
                                if meta.path.is_ident("into_from") {
                                    match meta.value()?.parse::<Expr>()? {
                                        Expr::Path(path) => into_from_path = Some(path.path),
                                        e => {
                                            return Err(syn::Error::new_spanned(
                                                e,
                                                "into_from must be a path",
                                            ));
                                        }
                                    }
                                }

                                Ok(())
                            });

                            if result.is_err() {
                                return result.unwrap_err().to_compile_error().into();
                            }
                        }

                        let ident = &field.ident;
                        if let Some(path) = into_from_path {
                            serialize.push(quote! {
                                cursor += bincode::encode_into_slice(
                                    &#path::from(self.#ident),
                                    &mut data[cursor..],
                                    bincode::config::standard(),
                                )?;
                            });
                            deserialize.push(quote! {
                                let (value, bytes_read): (#path, _) = bincode::decode_from_slice(
                                    &data[cursor..],
                                    bincode::config::standard(),
                                )?;
                                self.#ident = value.into();
                                cursor += bytes_read;
                            });
                        } else {
                            serialize.push(quote! {
                                cursor += bincode::encode_into_slice(
                                    &self.#ident,
                                    &mut data[cursor..],
                                    bincode::config::standard(),
                                )?;
                            });
                            deserialize.push(quote! {
                                let (value, bytes_read) = bincode::decode_from_slice(
                                    &data[cursor..],
                                    bincode::config::standard(),
                                )?;
                                self.#ident = value;
                                cursor += bytes_read;
                            });
                        }
                    }
                }
            }
        }
        _ => {
            return syn::Error::new_spanned(name, "Replicated can only be derived for structs")
                .to_compile_error()
                .into();
        }
    }

    if replication_id_field.is_none() {
        return syn::Error::new_spanned(name, "No field marked with #[replication_id]")
            .to_compile_error()
            .into();
    }

    let serialize = quote! { #(#serialize)* };
    let deserialize = quote! { #(#deserialize)* };

    let expanded = quote! {
        impl #impl_generics replication::Replicated for #name #ty_generics #where_clause {
            fn id(&self) -> replication::Id {
                self.#replication_id_field
            }

            fn component_id(&self, world: &bevy::ecs::world::World) -> bevy::ecs::component::ComponentId {
                world.component_id::<Self>().unwrap()
            }

            fn serialize(&self, data: &mut [u8]) -> ::anyhow::Result<usize> {
                let mut cursor = 0;
                #serialize;
                Ok(cursor)
            }

            fn replicate(&mut self, data: &[u8]) -> ::anyhow::Result<usize> {
                let mut cursor = 0;
                #deserialize;
                Ok(cursor)
            }
        }
    };

    expanded.into()
}
