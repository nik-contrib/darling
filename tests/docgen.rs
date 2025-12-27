use darling::{util::Flag, FromDeriveInput, FromMeta};
use pretty_assertions::assert_str_eq;
use quote::quote;

/// Flattened
#[derive(Default, FromMeta, PartialEq, Debug)]
struct Flattened {
    /// flattened_left
    flattened_left: String,
    /// flattened_right
    #[darling(multiple)]
    flattened_right: Vec<Lorem>,
}

#[derive(FromMeta, PartialEq, Debug)]
/// Enum
enum Enum {
    /// Enum::Unit
    Unit,
    /// Enum::Lorem
    Lorem(
        /// Enum::Lorem.0
        Lorem,
    ),
    /// Enum::Struct
    Struct {
        /// Enum::Struct.foo
        foo: bool,
        /// Enum::Struct.bar
        bar: Lorem,
    },
}

#[derive(Default, FromMeta, PartialEq, Debug)]
#[darling(default)]
/// LoremInner
struct LoremInner {
    /// LoremInner.inner
    inner: String,
}

#[derive(Default, FromMeta, PartialEq, Debug)]
#[darling(default)]
/// Lorem
struct Lorem {
    /// Lorem.ipsum
    ipsum: bool,
    /// Lorem.dolor
    dolor: Option<LoremInner>,
}

#[derive(FromDeriveInput, PartialEq, Debug)]
#[darling(attributes(demo))]
/// Input
struct Input {
    /// Input.ident
    ident: syn::Ident,
    /// Input.generics
    generics: syn::Generics,
    /// Input.lorem: #[derive(FromMeta)] struct
    lorem: Lorem,
    /// Input.multiple: #[darling(multiple)]
    #[darling(multiple)]
    multiple: Vec<String>,
    /// Input.multiple_lorem: #[darling(multiple)] on #[derive(FromMeta)] struct
    #[darling(multiple)]
    multiple_lorem: Vec<Lorem>,
    /// Input.foo: Enum
    foo: Enum,
    /// Input.not_present: This will be `None`
    not_present: Option<String>,
    // Input.flattened
    #[darling(flatten)]
    flattened: Flattened,
}

#[test]
fn docgen() {
    let input = quote! {
        #[demo(
            multiple_lorem(ipsum),
            multiple_lorem(dolor(inner = "foobar")),
            multiple = "a",
            multiple = "b",
            lorem(ipsum, dolor(inner = "foobar")),
            foo(struct(foo, bar(ipsum))),
            flattened_left = "left",
            flattened_right(ipsum),
            flattened_right(dolor(inner = "z")),
        )]
        struct Struct;
    };

    let input = Input::from_derive_input(&syn::parse2(input).unwrap()).unwrap();

    let expected_mods = quote! {
        #[doc = " Input"]
        pub mod Input {
            #[doc = " Input.lorem: #[derive(FromMeta)] struct"]
            pub mod lorem {
                #[doc = " Lorem.ipsum"]
                pub mod ipsum {}
                #[doc = " Lorem.dolor"]
                pub mod dolor {
                    #[doc = " LoremInner.inner"]
                    pub mod inner {}
                }
            }
            #[doc = " Input.multiple: #[darling(multiple)]"]
            pub mod multiple {}
            #[doc = " Input.multiple_lorem: #[darling(multiple)] on #[derive(FromMeta)] struct"]
            pub mod multiple_lorem {
                #[doc = " Lorem.ipsum"]
                pub mod ipsum {}
                #[doc = " Lorem.dolor"]
                pub mod dolor {
                    #[doc = " LoremInner.inner"]
                    pub mod inner {}
                }
            }
            #[doc = " Input.foo: Enum"]
            pub mod foo {
                #[doc = " Enum::Unit"]
                pub mod unit {}
                #[doc = " Enum::Lorem"]
                pub mod lorem {
                    #[doc = " Lorem.ipsum"]
                    pub mod ipsum {}
                    #[doc = " Lorem.dolor"]
                    pub mod dolor {
                        #[doc = " LoremInner.inner"]
                        pub mod inner {}
                    }
                }
                #[doc = " Enum::Struct"]
                pub mod r#struct {
                    #[doc = " Enum::Struct.foo"]
                    pub mod foo {}
                    #[doc = " Enum::Struct.bar"]
                    pub mod bar {
                        #[doc = " Lorem.ipsum"]
                        pub mod ipsum {}
                        #[doc = " Lorem.dolor"]
                        pub mod dolor {
                            #[doc = " LoremInner.inner"]
                            pub mod inner {}
                        }
                    }
                }
            }
            #[doc = " Input.not_present: This will be `None`"]
            pub mod not_present {}
            pub mod flattened {
                #[doc = " flattened_left"]
                pub mod flattened_left {}
                #[doc = " flattened_right"]
                pub mod flattened_right {
                    #[doc = " Lorem.ipsum"]
                    pub mod ipsum {}
                    #[doc = " Lorem.dolor"]
                    pub mod dolor {
                        #[doc = " LoremInner.inner"]
                        pub mod inner {}
                    }
                }
            }
        }
    };
    let expected_mods = expected_mods.to_string();
    // round-trip to "normalize" it so we can do string comparison
    let expected_mods = syn::parse_file(&expected_mods).unwrap();
    let expected_mods = quote!(#expected_mods).to_string();
    let actual_mods = Input::docs_mod().unwrap().to_string();
    // round-trip to "normalize" it so we can do string comparison
    let actual_mods = syn::parse_file(&actual_mods).unwrap();
    let actual_mods = quote!(#actual_mods).to_string();
    println!("{actual_mods}");
    assert_str_eq!(actual_mods, expected_mods.to_string());

    let docs_uses = input.docs_uses().unwrap();
    dbg!(&docs_uses);

    let expected_uses = quote! {
        use Input::{
            lorem::{
                ipsum as _,
                dolor::{inner as _,},
            },
            multiple as _,
            multiple_lorem::{
                ipsum as _,
                dolor as _,
                ipsum as _,
                dolor::{inner as _,},
            },
            foo::{
                r#struct::{
                    foo as _,
                    bar::{
                        ipsum as _,
                        dolor as _,
                    },
                },
            },
            not_present as _,
            flattened::{
                flattened_left as _,
                flattened_right::{
                    ipsum as _,
                    dolor as _,
                    ipsum as _,
                    dolor::{inner as _,},
                },
            },
        };
    };

    assert_str_eq!(quote!(#docs_uses).to_string(), expected_uses.to_string());

    panic!();
}
