use clap::Parser;
use handlebars::Handlebars;
use regex::Regex;
use std::{
    fs::{self, OpenOptions},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    str::FromStr,
};
use tera::{Context, Tera};

enum Template {
    Tera(String),
    Handlebars(String),
    Liquid(String),
    Unknown(String),
    Raw(String),
}

enum TemplateEngine {
    Tera,
    Handlebars,
    Unknown,
    NotSpecified,
}

// impl FromStr for TemplateEngine {
//     type Err = ();

//     fn from_str(template_str: &str) -> Result<Self, Self::Err> {
//         Ok(match template_str {
//             "tera" => Template::Tera(String::new()),
//             "handlebars" | "hbs" => Template::Handlebars(String::new()),
//             _ => Template::Unknown(String::new()),
//         })
//     }
// }

#[derive(Parser)]
#[clap(
    author = env!("CARGO_PKG_AUTHORS"),
    version = env!("CARGO_PKG_VERSION"),
    about = env!("CARGO_PKG_DESCRIPTION"),
    long_about = None
)]
struct Cli {
    /// A template file path, requiring a JSON file of the same name for context.
    /// e.g. `my_template.html` should have a context file named `my_template.ctx.json` in the same directory.
    #[clap(value_parser)]
    template_file: PathBuf,
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

fn load_template<P: AsRef<Path>>(path: P) -> Template {
    let template_contents = fs::read_to_string(&path).expect("Unable to load raw template.");

    // if let Some(stem) = path.as_ref().file_stem() {
    //     let stem = &*stem.to_string_lossy();
    //     println!("Stem: {stem}");
    // }

    if let Some(extension) = path.as_ref().extension() {
        let extension = &*extension.to_string_lossy();

        // println!("Extension: {extension}");

        match extension {
            "tera" => return Template::Tera(template_contents),
            "hbs" => return Template::Handlebars(template_contents),
            "liq" => return Template::Liquid(template_contents),
            _ => {}
        };
    }

    // TODO: Decode which type is the content and return it

    let re = Regex::new(r#"^<!--TEMPLATE\s+(?P<engine>\w+)-->"#).expect("Bad regex pattern.");

    let mut re_caps = re.captures_iter(&template_contents);

    if let Some(cap) = re_caps.next() {
        let engine = &cap["engine"];
        println!("Template Engine: {engine}");
        // TODO: Remove engine comment from final product before passing
        Template::Tera(template_contents)
    } else {
        Template::Raw(template_contents)
    }
}

fn main() {
    // When to use the `Tera` engine:
    // `Used to Jinja2, Django templates, Liquid or Twig? You will feel right at home.`
    // Want to use Jinja2, Django templates, Liquid or Twig? Use Tera (complete compatibly not guaranteed - More engine support may be added in the future)

    let args = Cli::parse();

    let template_extension = &*args
        .template_file
        .extension()
        .expect("Template has no file extension.")
        .to_string_lossy();

    let template_extension = "rendered.".to_string() + template_extension;

    let template_context_file = args.template_file.with_extension("ctx.json");

    let context_json =
        fs::read_to_string(&template_context_file).expect("Unable to load template context.");

    let context: serde_json::Value =
        serde_json::from_str(&context_json).expect("Unable to parse context JSON.");

    let rendered_output_file = args.template_file.with_extension(&template_extension);

    // #[cfg(build = "debug")]
    println!("Rendering File: {}", args.template_file.to_string_lossy());
    // #[cfg(build = "debug")]
    println!("Context File: {}", template_context_file.to_string_lossy());
    // #[cfg(build = "debug")]
    println!(
        "Rendered Output File: {}",
        rendered_output_file.to_string_lossy()
    );

    let template_contents = load_template(args.template_file);

    let result = match template_contents {
        Template::Tera(contents) => {
            let context =
                Context::from_value(context).expect("Unable to create context from JSON.");

            Tera::one_off(&contents, &context, true).expect("Unable to render template.")
        }
        Template::Handlebars(contents) => {
            let handlebars = Handlebars::new();
            handlebars
                .render_template(&contents, &context)
                .expect("Unable to render template.")
        }
        Template::Liquid(contents) => todo!(),
        Template::Unknown(_) => panic!("Unknown template engine."),
        Template::Raw(raw) => raw,
    };

    output_render(&result, rendered_output_file);
}
