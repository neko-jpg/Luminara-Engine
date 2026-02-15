use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};
use quote::quote;

#[proc_macro_derive(DbModel, attributes(db))]
pub fn derive_db_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Stub implementation
    let gen = quote! {
        impl #name {
            pub fn type_name() -> &'static str {
                stringify!(#name)
            }
        }
    };

    gen.into()
}
