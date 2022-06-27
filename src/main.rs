#[macro_use]
extern crate lazy_static;

use std::{fs, path::PathBuf};

lazy_static! {
    static ref TEMPLATES_DIR: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates");
}

fn load_template(name: &str) -> String {
    let mut template_path = TEMPLATES_DIR.join(name);
    template_path.set_extension("html");
    println!("Loading `{}`", template_path.to_string_lossy());
    fs::read_to_string(template_path).expect("Unable to load template contents")
}

fn main() {
    let template = "test";
    let template_contents = load_template(template);
    println!("{template_contents}")
}
