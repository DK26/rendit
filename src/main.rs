use bat::PrettyPrinter;
use clap::Parser;
use handlebars::{Handlebars, TemplateError};
use regex::RegexBuilder;
use std::{
    error::Error,
    fs::{self, OpenOptions},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};
use tera::{Context, Tera};

enum Template {
    Tera(String),
    Handlebars(String),
    Liquid(String),
    Unknown(String, String),
    NoEngine(String),
}

#[derive(Parser)]
#[clap(
    author = env!("CARGO_PKG_AUTHORS"),
    version = env!("CARGO_PKG_VERSION"),
    about = env!("CARGO_PKG_DESCRIPTION"),
    long_about = None
)]
struct Cli {
    /// A template file path, requiring a JSON file of the same name for context.
    ///
    /// e.g. `my_template.html` should have a context file named `my_template.ctx.json` in the same directory.
    #[clap(value_parser)]
    template_file: PathBuf,

    /// Sets the level of verbosity.
    ///  
    /// `-v` sets logging level to INFO
    /// `-vv` sets logging level to DEBUG
    #[clap(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

// TODO: STDIN support
// TODO: By default render to stdout + Use `bat` for a Pretty Print
// TODO: Add write to file option `--output <FILE PATH>`, `--output <FILE PATH>, stdout`
// TODO: Add custom context file `--json-context`, `--context-file`, `--context <FILE PATH>`

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

impl From<String> for Template {
    /// Inspect the String contents for a magic comment `<!--template engine_name-->`, and return the appropriate `Template` enum variation for rendering.
    fn from(contents: String) -> Self {
        let re = RegexBuilder::new(r#"^(?:\s+)?<!--template\s+(?P<engine>\w+)\s?-->"#)
            .case_insensitive(true)
            .build()
            .expect("Bad regex pattern.");

        let mut re_caps = re.captures_iter(&contents);

        if let Some(m) = re.find(&contents) {
            let found_match = m.as_str();

            let contents = contents.replacen(found_match, "", 1).trim().to_owned();

            let cap = re_caps
                .next()
                .expect("Match without a capture? how is it possible?");

            let engine = cap["engine"].to_lowercase();

            log::debug!("Detected Engine: `{engine}`");

            match engine.as_str() {
                "tera" => Template::Tera(contents),
                "hbs" | "handlebars" => Template::Handlebars(contents),
                "liq" | "liquid" => Template::Liquid(contents),
                unknown_engine => Template::Unknown(unknown_engine.to_owned(), contents),
            }
        } else {
            Template::NoEngine(contents)
        }
    }
}

/// Loads a template file into a Template enum type.
/// Decides on the engine type by first inspecting the file extension (`.tera`, `.hbs` or `.liq`).
/// If no special extension is provided then the contents of the template are inspected for the magic comment `<!--TEMPLATE engine_name-->`.
///
/// Engine Names: `tera`, `handlebars` or `hbs`, `liquid` or `liq`
fn load_template_file<P: AsRef<Path>>(path: P) -> Template {
    let template_contents = fs::read_to_string(&path).expect("Unable to load raw template.");

    // if let Some(stem) = path.as_ref().file_stem() {
    //     let stem = &*stem.to_string_lossy();
    //     println!("Stem: {stem}");
    // }

    if let Some(extension) = path.as_ref().extension() {
        let file_extension = &*extension.to_string_lossy();

        match file_extension {
            "tera" => return Template::Tera(template_contents),
            "hbs" => return Template::Handlebars(template_contents),
            "liq" => return Template::Liquid(template_contents),
            _ => {} // ignore unknown extensions
        };
    }

    // Scan template contents for the magic comment to return the proper Template kind.
    template_contents.into()
}

fn pretty_print(content: &str, extension: Option<&str>) {
    let bytes_content = content.as_bytes();
    PrettyPrinter::new()
        .language(extension.unwrap_or("html")) // Default: auto-detect
        .line_numbers(false)
        .grid(true)
        .header(true)
        .input(bat::Input::from_bytes(bytes_content))
        .print()
        .expect("Unable to pretty print.");
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

    let rendered_template_extension = "rendered.".to_string() + template_extension;

    let template_context_file = args.template_file.with_extension("ctx.json");

    let context_json =
        fs::read_to_string(&template_context_file).expect("Unable to load template context.");

    let context: serde_json::Value =
        serde_json::from_str(&context_json).expect("Unable to parse context JSON.");

    let rendered_output_file = args
        .template_file
        .with_extension(&rendered_template_extension);

    log::info!("Rendering File: {}", args.template_file.to_string_lossy());
    log::info!("Context File: {}", template_context_file.to_string_lossy());
    log::info!(
        "Rendered Output File: {}",
        rendered_output_file.to_string_lossy()
    );

    let template_contents = load_template_file(&args.template_file);

    let result = match template_contents {
        Template::Tera(contents) => {
            let context =
                Context::from_value(context).expect("Unable to create context from JSON.");

            match Tera::one_off(&contents, &context, true) {
                Ok(rendered) => rendered,
                Err(e) => {
                    if let Some(source) = e.source() {
                        log::error!("{source}");
                        // eprintln!("{source}");
                    }

                    panic!("Unable to render template.");
                }
            }
        }
        Template::Handlebars(contents) => {
            let handlebars = Handlebars::new();
            let render = handlebars.render_template(&contents, &context);
            match render {
                Ok(contents) => contents,
                Err(e) => {
                    if let Some(source) = e.source() {
                        if let Some(template_error) = source.downcast_ref::<TemplateError>() {
                            let template_error_string = format!("{template_error}");
                            pretty_print(&template_error_string, Some(template_extension));
                            // eprintln!("{template_error}");
                        }
                    }
                    panic!("Unable to render template.");
                }
            }
        }
        Template::Liquid(contents) => {
            let template = liquid::ParserBuilder::with_stdlib()
                .build()
                .expect("Unable to build Liquid parser.")
                .parse(&contents);

            let template = match template {
                Ok(t) => t,
                Err(e) => {
                    let template_error_string = format!("{e}");
                    pretty_print(&template_error_string, Some(template_extension));
                    // eprintln!("{e}");
                    panic!("Unable to parse template.");
                }
            };

            let globals = liquid::object!(&context);

            template
                .render(&globals)
                .expect("Unable to render template.")
        }
        Template::Unknown(engine, _) => panic!("Unknown template engine: `{engine}`"),
        Template::NoEngine(raw) => raw,
    };

    // Cancelled: PrettyPrint -> Characters are not standard and cannot be redirected properly with pipes.. for now.
    // PrettyPrinter::new()
    //     .language("html") // Default: auto-detect
    //     .line_numbers(true)
    //     .grid(true)
    //     .header(true)
    //     .input(bat::Input::from_bytes(result.as_bytes()))
    //     .print()
    //     .unwrap();
    println!("{result}");

    // TODO: Output to file only if output argument is given
    output_render(&result, rendered_output_file);
}
