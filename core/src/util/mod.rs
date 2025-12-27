//! Utility types for attribute parsing.

mod callable;
mod flag;
mod ident_string;
mod ignored;
mod over_ride;
mod parse_attribute;
pub mod parse_expr;
mod path_list;
mod path_to_string;
mod preserved_str_expr;
mod shape;
mod spanned_value;
mod with_original;

#[doc(hidden)]
pub fn safe_ident(s: &'static str) -> syn::Ident {
    const STRICT_KEYWORDS: &[&str] = &[
        "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum",
        "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move",
        "mut", "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait",
        "true", "type", "unsafe", "use", "where", "while",
    ];

    if STRICT_KEYWORDS.contains(&s) {
        syn::Ident::new_raw(s, Span::call_site())
    } else {
        syn::Ident::new(s, Span::call_site())
    }
}

use proc_macro2::Span;

pub use self::callable::Callable;
pub use self::flag::Flag;
pub use self::ident_string::IdentString;
pub use self::ignored::Ignored;
pub use self::over_ride::Override;
pub use self::parse_attribute::parse_attribute_to_meta_list;
pub use self::path_list::PathList;
pub use self::path_to_string::path_to_string;
pub use self::preserved_str_expr::PreservedStrExpr;
pub use self::shape::{AsShape, Shape, ShapeSet};
pub use self::spanned_value::SpannedValue;
pub use self::with_original::WithOriginal;
