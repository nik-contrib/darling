use darling::{FromDeriveInput, FromMeta};

#[derive(FromMeta, PartialEq, Debug)]
/// Lorem
enum Enum {
    /// Unit: Enum variant with no fields
    Unit,
    /// Lorem: Newtype enum variant
    Lorem(Lorem),
    /// Struct: Enum variant with fields
    Struct { foo: bool, bar: Lorem },
}

#[derive(Default, FromMeta, PartialEq, Debug)]
#[darling(default)]
/// LoremInner
struct LoremInner {
    /// inner
    inner: String,
}

#[derive(Default, FromMeta, PartialEq, Debug)]
#[darling(default)]
/// Lorem
struct Lorem {
    /// ipsum
    ipsum: bool,
    /// dolor
    dolor: Option<LoremInner>,
}

#[derive(FromDeriveInput, PartialEq, Debug)]
#[darling(attributes(demo))]
/// Input
struct Input {
    /// ident
    ident: syn::Ident,
    /// generics
    generics: syn::Generics,
    /// lorem: #[derive(FromMeta)] struct
    lorem: Lorem,
    /// multiple: #[darling(multiple)]
    #[darling(multiple)]
    multiple: Vec<String>,
    /// multiple: #[darling(multiple)] on #[derive(FromMeta)] struct
    #[darling(multiple)]
    multiple_lorem: Vec<Lorem>,
    /// foo: Enum
    foo: Enum,
}

#[test]
fn docgen() {
    let input = Input::docs_mod().unwrap();

    println!("{input}");
    panic!();
}
