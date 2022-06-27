#[macro_use]
extern crate lazy_static;

use std::{fs, path::PathBuf};

use tera::{Context, Tera};

lazy_static! {
    static ref TEMPLATES_DIR: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates");
}

fn load_template(name: &str) -> String {
    let mut template_path = TEMPLATES_DIR.join(name);
    template_path.set_extension("html");
    println!("Loading `{}`", template_path.to_string_lossy());
    fs::read_to_string(template_path).expect("Unable to load raw template.")
}

fn load_context(name: &str) -> Context {
    let mut template_path = TEMPLATES_DIR.join(name);
    template_path.set_extension("json");

    let context_json = fs::read_to_string(template_path).expect("Unable to load template context.");

    Context::from_value(context_json.parse().expect("Unable to parse context JSON."))
        .expect("Unable to create context from JSON.")
}

fn main() {
    // When to use the `Tera` engine:
    // `Used to Jinja2, Django templates, Liquid or Twig? You will feel right at home.`
    // Want to use Jinja2, Django templates, Liquid or Twig? Use Tera (complete compatibly not guaranteed - More engine support may be added in the future)

    let template = "test";
    let template_contents = load_template(template);
    // println!("{template_contents}");

    // let mut context = Context::new();
    // context.insert("user", &"D.K");

    let context = load_context(template);

    let result =
        Tera::one_off(&template_contents, &context, true).expect("Unable to render template.");

    println!("{result}");
}
