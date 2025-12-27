use std::borrow::Cow;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::Ident;

use crate::ast::Fields;
use crate::codegen::error::{ErrorCheck, ErrorDeclaration};
use crate::codegen::{Field, FieldsGen};
use crate::usage::{self, IdentRefSet, IdentSet, UsesTypeParams};

/// A variant of the enum which is deriving `FromMeta`.
#[derive(Debug, Clone)]
pub struct Variant<'a> {
    /// The name which will appear in code passed to the `FromMeta` input.
    pub name_in_attr: Cow<'a, str>,

    /// The name of the variant which will be returned for a given `name_in_attr`.
    pub variant_ident: &'a Ident,

    /// The name of the parent enum type.
    pub ty_ident: &'a Ident,

    pub data: Fields<Field<'a>>,

    /// Whether or not the variant should be skipped in the generated code.
    pub skip: bool,

    pub allow_unknown_fields: bool,

    pub docs: &'a [String],
}

impl<'a> Variant<'a> {
    pub fn as_name(&'a self) -> &'a str {
        &self.name_in_attr
    }

    pub fn as_unit_match_arm(&'a self) -> UnitMatchArm<'a> {
        UnitMatchArm(self)
    }

    pub fn as_data_match_arm(&'a self) -> DataMatchArm<'a> {
        DataMatchArm(self)
    }

    pub fn as_docs_mod(&'a self) -> DocsMod<'a> {
        DocsMod(self)
    }

    pub fn as_docs_uses(&'a self) -> DocsUses<'a> {
        DocsUses(self)
    }
}

impl UsesTypeParams for Variant<'_> {
    fn uses_type_params<'b>(
        &self,
        options: &usage::Options,
        type_set: &'b IdentSet,
    ) -> IdentRefSet<'b> {
        self.data.uses_type_params(options, type_set)
    }
}

/// Code generator for an enum variant in a unit match position.
/// This is placed in generated `from_string` calls for the parent enum.
/// Value-carrying variants wrapped in this type will emit code to produce an "unsupported format" error.
pub struct UnitMatchArm<'a>(&'a Variant<'a>);

impl ToTokens for UnitMatchArm<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let val: &Variant<'_> = self.0;

        if val.skip {
            return;
        }

        let name_in_attr = &val.name_in_attr;

        let unsupported_format_error = || {
            quote!(::darling::export::Err(
                ::darling::Error::unsupported_format("literal")
            ))
        };

        if val.data.is_unit() {
            let variant_ident = val.variant_ident;
            let ty_ident = val.ty_ident;

            tokens.append_all(quote!(
                #name_in_attr => ::darling::export::Ok(#ty_ident::#variant_ident),
            ));
        } else if let Some(field) = val.data.as_newtype() {
            let field_ty = field.ty;
            let ty_ident = val.ty_ident;
            let variant_ident = val.variant_ident;
            let unsupported_format = unsupported_format_error();

            tokens.append_all(quote!{
                #name_in_attr => {
                    match <#field_ty as ::darling::FromMeta>::from_none() {
                        ::darling::export::Some(__value) => ::darling::export::Ok(#ty_ident::#variant_ident(__value)),
                        ::darling::export::None => #unsupported_format,
                    }
                }
            })
        } else {
            let unsupported_format = unsupported_format_error();
            tokens.append_all(quote!(
                #name_in_attr => #unsupported_format,
            ));
        }
    }
}

/// Code generator for an enum variant in a data-carrying match position.
/// This is placed in generated `from_list` calls for the parent enum.
/// Unit variants wrapped in this type will emit code to produce an "unsupported format" error.
pub struct DataMatchArm<'a>(&'a Variant<'a>);

impl ToTokens for DataMatchArm<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let val: &Variant<'_> = self.0;

        if val.skip {
            return;
        }

        let name_in_attr = &val.name_in_attr;
        let variant_ident = val.variant_ident;
        let ty_ident = val.ty_ident;

        if val.data.is_unit() {
            // Allow unit variants to match a list item if it's just a path with no associated
            // value, e.g. `volume(shout)` is allowed.
            tokens.append_all(quote!(
                #name_in_attr => {
                    if let ::darling::export::syn::Meta::Path(_) = *__nested {
                        ::darling::export::Ok(#ty_ident::#variant_ident)
                    } else {
                        ::darling::export::Err(::darling::Error::unsupported_format("non-path"))
                    }
                },
            ));

            return;
        }

        let vdg = FieldsGen::new(&val.data, val.allow_unknown_fields);

        if val.data.is_struct() {
            let declare_errors = ErrorDeclaration::default();
            let check_errors = ErrorCheck::with_location(name_in_attr);
            let require_fields = vdg.require_fields();
            let decls = vdg.declarations();
            let core_loop = vdg.core_loop();
            let inits = vdg.initializers();

            tokens.append_all(quote!(
                #name_in_attr => {
                    if let ::darling::export::syn::Meta::List(ref __data) = *__nested {
                        let __items = ::darling::export::NestedMeta::parse_meta_list(__data.tokens.clone())?;
                        let __items = &__items;

                        #declare_errors

                        #decls

                        #core_loop

                        #require_fields

                        #check_errors

                        ::darling::export::Ok(#ty_ident::#variant_ident {
                            #inits
                        })
                    } else {
                        ::darling::export::Err(::darling::Error::unsupported_format("non-list"))
                    }
                }
            ));
        } else if val.data.is_newtype() {
            tokens.append_all(quote!(
                #name_in_attr => {
                    ::darling::export::Ok(
                        #ty_ident::#variant_ident(
                            ::darling::FromMeta::from_meta(__nested)
                                .map_err(|e| e.at(#name_in_attr))?)
                    )
                }
            ));
        } else {
            panic!("Match arms aren't supported for tuple variants yet");
        }
    }
}

/// Code generator for an enum variant in a data-carrying match position.
/// This is placed in generated `from_list` calls for the parent enum.
/// Unit variants wrapped in this type will emit code to produce an "unsupported format" error.
pub struct DocsMod<'a>(&'a Variant<'a>);

impl ToTokens for DocsMod<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let variant = self.0;

        if variant.skip {
            return;
        }

        let name = &variant.name_in_attr;
        let docs = &variant.docs;

        let children = if variant.data.is_empty() {
            quote! { ::darling::export::Vec::new() }
        } else if let Some(inner) = variant.data.as_newtype() {
            let ty = &inner.ty;
            quote! { <#ty as ::darling::FromMeta>::docs_mods() }
        } else if variant.data.is_struct() {
            let vdg = FieldsGen::new(&variant.data, variant.allow_unknown_fields);
            let docs_mod = vdg.docs_mod();
            quote! { ::darling::export::Vec::from([#docs_mod]) }
        } else {
            unreachable!()
        };

        tokens.append_all(quote!(
            ::darling::DocsMod {
                docs: ::darling::export::Vec::from([
                    #(::darling::export::String::from(#docs),)*
                ]),
                name: ::darling::util::safe_ident(#name, ::darling::export::Span::call_site()),
                children: #children
            },
        ));
    }
}

/// Code generator for an enum variant in a data-carrying match position.
/// This is placed in generated `from_list` calls for the parent enum.
/// Unit variants wrapped in this type will emit code to produce an "unsupported format" error.
pub struct DocsUses<'a>(&'a Variant<'a>);

impl ToTokens for DocsUses<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let variant = self.0;

        if variant.skip {
            return;
        }

        let name = &variant.name_in_attr;
        let ident = &variant.variant_ident;

        let (destructure, children) = if variant.data.is_empty() {
            (quote! {}, quote! { ::darling::export::Vec::new() })
        } else if let Some(inner) = variant.data.as_newtype() {
            let ty = inner.ty;
            let ident = inner.ident;
            (
                quote! { (#ident) },
                quote! { <#ty as ::darling::FromMeta>::docs_uses(&#ident) },
            )
        } else if variant.data.is_struct() {
            let fields = variant.data.iter().map(|field| &field.ident);
            let vdg = FieldsGen::new(&variant.data, variant.allow_unknown_fields);
            let docs_uses = vdg.docs_uses(true);
            (
                quote! { { #(#fields,)* } },
                quote! {{
                    let mut docs_uses = ::darling::export::Vec::new();
                    #(
                        ::darling::export::Extend::extend(&mut docs_uses, #docs_uses);
                    )*
                    docs_uses
                }},
            )
        } else {
            unreachable!()
        };

        tokens.append_all(quote!(
            Self::#ident #destructure => ::darling::export::Vec::from([::darling::DocsUses {
                parent: ::darling::util::safe_ident(#name),
                children: #children
            }]),
        ));
    }
}
