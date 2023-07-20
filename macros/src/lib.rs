use proc_macro::TokenStream;
use quote::quote;
use syn;
use syn::DeriveInput;

#[proc_macro_derive(IntoJsonResponse)]
pub fn into_json_response(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let struct_name = ast.ident;

    (quote! {
        impl axum::response::IntoResponse for #struct_name {
            fn into_response(self) -> axum::response::Response {
                axum::Json(self).into_response()
            }
        }
    }).into()
}
