use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use std::fmt::{self, Write};
use syn::{parse_quote, Ident, ItemUse};

use crate::FromDeriveInput;

pub struct DocsMod {
    /// A list of `///` comments
    pub docs: Vec<String>,
    /// Name of the `mod`, e.g. `mod name`
    pub name: Ident,
    /// Children modules, each with documentation attached
    pub children: Vec<DocsMod>,
}

impl DocsMod {
    pub fn new(name: &'static str) -> Self {
        Self {
            docs: Vec::new(),
            name: Ident::new(name, Span::call_site()),
            children: Vec::new(),
        }
    }
}

/// Creates the `mod` containing documentation comments attached, and
/// all children modules
impl fmt::Display for DocsMod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for doc in &self.docs {
            writeln!(f, "///{doc}")?;
        }

        write!(f, "pub mod {} {{", self.name)?;

        if self.children.is_empty() {
            // no children, close to make it `mod foo {}`
            writeln!(f, "}}")?;
            return Ok(());
        } else {
            writeln!(f)?;
        }

        let children_len = self.children.len();
        for (i, child) in self.children.iter().enumerate() {
            for line in child.to_string().lines() {
                if line.is_empty() {
                    // with an extra level of indentation
                    writeln!(f)?;
                } else {
                    // with an extra level of indentation
                    writeln!(f, "    {line}")?;
                }
            }

            // Skip adding a newline at the end
            if i + 1 != children_len {
                writeln!(f)?;
            }
        }

        writeln!(f, "}}")?;

        Ok(())
    }
}

/// Call `.generate(syn::parse_quote!(...))`, where `...` is the path where the top-level [`DocsMod`] lives
/// and then output that as part of your macro's final output `TokenStream`
#[derive(Default)]
pub struct DocsUses(Vec<syn::Path>);

impl Extend<DocsUses> for DocsUses {
    fn extend<T: IntoIterator<Item = DocsUses>>(&mut self, iter: T) {
        for i in iter {
            self.0.extend(i.0);
        }
    }
}

impl DocsUses {
    pub fn new(path: &'static str) -> Self {
        let ident = syn::Ident::new(path, Span::call_site());
        Self(Vec::from([parse_quote!(#ident)]))
    }

    pub fn generate(self, root: syn::Path) -> TokenStream {
        let paths = &self.0;
        let root = &root;
        quote! {
            #(use #root::#paths as _)*
        }
    }
}

/// A `T` and its documentation imports
pub struct Documented<T> {
    /// Any item that can have documentation attached
    pub item: T,
    /// Documentation for `item`
    pub docs: DocsUses,
}

impl<T: FromDeriveInput> FromDeriveInput for Documented<T> {
    fn from_derive_input(input: &syn::DeriveInput) -> crate::Result<Self> {
        let item = T::from_derive_input(input)?;
        Ok(Documented {
            docs: item.docs_uses(),
            item,
        })
    }
}
