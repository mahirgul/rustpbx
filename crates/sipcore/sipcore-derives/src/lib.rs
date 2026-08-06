use proc_macro::TokenStream;

#[proc_macro_derive(TypedHeader)]
pub fn derive_typed_header(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}
