use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use std::fmt::{self};
use syn::Ident;

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

/// This is like `Vec<syn::Path>`, but we don't unnecessarily keep
/// more identifiers than necessary
///
/// When `children` is empty, this is just a single identifier. e.g. `foo`
///
/// When children is non-empty, this is a list of paths, e.g. if children is `[foo::quux, bar]` and parent is `baz`, then
/// this is equivalent to 2 paths: `baz::foo::quux` and `baz::bar`
pub struct DocsUses {
    pub parent: syn::Ident,
    pub children: Vec<DocsUses>,
}

impl DocsUses {
    fn write_use_tree(&self, tokens: &mut TokenStream) {
        let parent = &self.parent;
        if self.children.is_empty() {
            tokens.extend(quote! { #parent as _ });
        } else {
            let mut children_use_trees = TokenStream::new();
            for child in &self.children {
                child.write_use_tree(&mut children_use_trees);
                children_use_trees.extend(quote! {,});
            }
            tokens.extend(quote! { #parent::{#children_use_trees} });
        }
    }
}

impl ToTokens for DocsUses {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(quote!(use));
        self.write_use_tree(tokens);
        tokens.extend(quote!(;));
    }
}

// impl DocsUse {
//     /// Create a single identifier
//     pub fn new(path: &'static str, span: impl syn::spanned::Spanned) -> Self {
//         Self {
//             parent: syn::Ident::new(path, span.span()),
//             children: None,
//         }
//     }

//     // pub fn generate(self, root: syn::Path) -> TokenStream {
//     //     let paths = &self.0;
//     //     let root = &root;
//     //     quote! {
//     //         #(use #root::#paths as _)*
//     //     }
//     // }
// }
