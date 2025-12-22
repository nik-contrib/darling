use darling::FromDeriveInput;

#[derive(FromDeriveInput)]
/// This is my input
struct Input {}

fn main() {
    Input::docs_mod().unwrap();
}
