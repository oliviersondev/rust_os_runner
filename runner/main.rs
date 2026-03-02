use std::env;

fn main() {
    let iso = env::var("ISO").unwrap();
    println!("ISO path: {iso:?}");
}
