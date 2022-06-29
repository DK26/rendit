use clap::Parser;
use std::{
    fs::{self, OpenOptions},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};
use tera::{Context, Tera};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(value_parser)]
    template_file: PathBuf,
}

// #[macro_use]
// extern crate lazy_static;

// #[cfg(build = "debug")]
// lazy_static! {
//     static ref TEMPLATES_DIR: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates");
// }

fn load_context<P: AsRef<Path>>(path: P) -> Context {
    let context_json = fs::read_to_string(path).expect("Unable to load template context.");

    Context::from_value(context_json.parse().expect("Unable to parse context JSON."))
        .expect("Unable to create context from JSON.")
}

fn output_render<P: AsRef<Path>>(content: &str, path: P) {
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(path)
        .expect("Unable to create file");

    let mut bw = BufWriter::new(file);
    bw.write_all(content.as_bytes())
        .expect("Unable to write rendered HTML");
}

fn load_template<P: AsRef<Path>>(path: P) -> TemplateEngine {
    let template_contents = fs::read_to_string(path).expect("Unable to load raw template.");
    TemplateEngine::Tera(template_contents)
}

enum TemplateEngine {
    Tera(String),
}

fn main() {
    // When to use the `Tera` engine:
    // `Used to Jinja2, Django templates, Liquid or Twig? You will feel right at home.`
    // Want to use Jinja2, Django templates, Liquid or Twig? Use Tera (complete compatibly not guaranteed - More engine support may be added in the future)

    let args = Cli::parse();

    let template_context_file = args.template_file.with_extension("json");
    let rendered_output_file = args.template_file.with_extension("rendered.html");

    println!("Rendering File: {}", args.template_file.to_string_lossy());
    println!("Context File: {}", template_context_file.to_string_lossy());
    println!(
        "Rendered Output File: {}",
        rendered_output_file.to_string_lossy()
    );

    let template_contents = load_template(args.template_file);

    let result = match template_contents {
        TemplateEngine::Tera(contents) => {
            Tera::one_off(&contents, &load_context(template_context_file), true)
                .expect("Unable to render template.")
        }
    };

    output_render(&result, rendered_output_file);
}
