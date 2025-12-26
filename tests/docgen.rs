use darling::FromDeriveInput;

#[derive(FromDeriveInput)]
/// This is my input
struct Input {}

#[test]
fn docgen() {
    let input = Input::docs_mod().unwrap();

    println!("{input}");
    panic!();
}
