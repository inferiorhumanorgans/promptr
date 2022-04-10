use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(SerializeNonDefault)]
pub fn only_serialize_non_default(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let name_string = name.to_string();

    let mut serialize_fields = vec![];

    if let syn::Data::Struct(st) = input.data {
        for field in st.fields.iter() {
            if let Some(ident) = &field.ident {
                let ident_s = ident.to_string();
                serialize_fields.push(quote! {
                    if self.#ident != default.#ident {
                        state.serialize_field(#ident_s, &self.#ident)?
                    }
                })
            }
        }
    }

    let serialize_count = serialize_fields.len();

    let quoted = quote! {

        impl Serialize for #name
        {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer + Sized,
            {
                let default = Self::default();
                let mut state = serializer.serialize_struct(#name_string, #serialize_count)?;

                #(#serialize_fields)*

                state.end()
            }
        }
    };

    TokenStream::from(quoted)
}
