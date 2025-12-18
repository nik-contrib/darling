use quote::{quote, ToTokens, TokenStreamExt};
use syn::{Ident, ItemUse};

use crate::FromDeriveInput;

pub struct DocsModule {
    /// A list off `///` comments
    docs: Vec<String>,
    /// Name of the `mod`, e.g. `mod name`
    name: Ident,
    /// Children modules, each with documentation attached
    children: Vec<DocsModule>,
}

pub struct DocsImports(Vec<syn::Path>);

impl DocsImports {
    pub fn none() -> Self {
        Self(Vec::new())
    }
}

impl ToTokens for DocsImports {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let docs = &self.0;
        tokens.append_all([quote! {
            const _: () = {
                use #(#docs)* as _;
            };
        }]);
    }
}

pub struct Documented<T> {
    item: T,
    /// Documentation for `item`
    docs: DocsImports,
}

impl<T: ToTokens> ToTokens for Documented<T> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.append_all([&self.item]);
        self.docs.to_tokens(tokens);
    }
}

impl<T: FromDeriveInput> FromDeriveInput for Documented<T> {
    fn from_derive_input(input: &syn::DeriveInput) -> crate::Result<Self> {
        Ok(T::from_derive_input(input))
    }
}
