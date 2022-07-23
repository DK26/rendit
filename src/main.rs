use anyhow::Result;
use bat::PrettyPrinter;
use clap::Parser;
use handlebars::{Handlebars, TemplateError};
use log::LevelFilter;
use regex::RegexBuilder;
use simplelog::TermLogger;
use std::{
    error::Error,
    fs::{self, OpenOptions},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};
use tera::{Context, Tera};

type Contents = String;
type EngineName = String;

enum TemplateKind {
    Tera(Contents),
    Handlebars(Contents),
    Liquid(Contents),
    Unknown(EngineName, Contents),
    NoEngine(Contents),
}

#[derive(Parser)]
#[clap(
    author = env!("CARGO_PKG_AUTHORS"),
    version = env!("CARGO_PKG_VERSION"),
    about = env!("CARGO_PKG_DESCRIPTION"),
    long_about = None
)]
struct Cli {
    /// The template file path, requiring a `default.ctx.json` context file or template specific context file
    /// containing the template name, ending with the `.ctx.json` extension.
    ///
    /// e.g. for `my_template.html`, a specific context file name will be `my_template.ctx.json`, located under the same directory.
    ///
    /// NOTICE: Providing `<TEMPLATE FILE>` file, produces a default rendered output file with the proper extension `<TEMPLATE FILE>.rendered.<extension>`.
    ///
    /// NOTICE: By NOT providing `<TEMPLATE FILE>`, the CLI will attempt to read the template data from STDIN, WITHOUT producing a default `.rendered.<extension>` file.
    #[clap(value_parser, value_name = "TEMPLATE FILE")]
    template_file: Option<PathBuf>,

    /// Override default context files with specified context file.
    #[clap(
        value_parser,
        short,
        long = "context-file",
        value_name = "CONTEXT FILE"
    )]
    context_file: Option<PathBuf>,

    #[clap(value_parser, short, long = "output", value_name = "OUTPUT FILE")]
    output_file: Option<PathBuf>,

    /// Sets the level of verbosity.
    ///  
    /// `-v` sets logging level to INFO
    /// `-vv` sets logging level to DEBUG
    ///
    /// WARNING: Effects CLI output.
    /// Sse `--output` switch if you wish to commit the rendered output to file.
    #[clap(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Print pretty, highlighted output to the terminal.
    ///
    /// WARNING: CLI output cannot be used for piping as ASCII/UTF-8 is transformed.
    /// Use `--output` switch if you wish to commit the rendered output to file.
    #[clap(short, long, action)]
    pretty: bool,

    /// Open rendered output file
    #[clap(short = 'P', long, action)]
    preview: bool,

    /// Print rendered result to STDOUT
    #[clap(short, long, action)]
    stdout: bool,

    /// Constantly render changes in template file.  
    #[clap(short, long = "watch", value_name = "SECONDS", action)]
    watch_seconds: Option<u8>,
}

/// Write `content` to file `path` using BufWriter
fn write_to_file<P: AsRef<Path>>(content: &str, path: P) {
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(path)
        .expect("Unable to create file");

    let mut bw = BufWriter::new(file);
    bw.write_all(content.as_bytes())
        .expect("Unable to write rendered HTML");
}

impl From<String> for TemplateKind {
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
                "tera" => TemplateKind::Tera(contents),
                "hbs" | "handlebars" => TemplateKind::Handlebars(contents),
                "liq" | "liquid" => TemplateKind::Liquid(contents),
                unknown_engine => TemplateKind::Unknown(unknown_engine.to_owned(), contents),
            }
        } else {
            TemplateKind::NoEngine(contents)
        }
    }
}

/// Loads a template file into a Template enum type.
/// Decides on the engine type by first inspecting the file extension (`.tera`, `.hbs` or `.liq`).
/// If no special extension is provided then the contents of the template are inspected for the magic comment `<!--TEMPLATE engine_name-->`.
///
/// Engine Names: `tera`, `handlebars` or `hbs`, `liquid` or `liq`
fn load_template_file<P: AsRef<Path>>(path: P) -> TemplateKind {
    let template_contents = fs::read_to_string(&path).expect("Unable to load raw template.");

    // if let Some(stem) = path.as_ref().file_stem() {
    //     let stem = &*stem.to_string_lossy();
    //     println!("Stem: {stem}");
    // }

    if let Some(extension) = path.as_ref().extension() {
        let file_extension = &*extension.to_string_lossy();

        match file_extension {
            "tera" => return TemplateKind::Tera(template_contents),
            "hbs" => return TemplateKind::Handlebars(template_contents),
            "liq" => return TemplateKind::Liquid(template_contents),
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

fn stdin_read() -> String {
    let lines = std::io::stdin().lines();

    lines
        .map(|line| {
            let l = line.expect("Failed to read stdin line");
            l + "\n"
        })
        .collect()
}

fn main() -> Result<()> {
    let args = Cli::parse();

    // TODO: By default render to stdout + Use `bat` for a Pretty Print (optional)

    if let Some(_template_file) = &args.template_file {
    } else {
        let template_contents = stdin_read();
    }

    // Checks if `<TEMPLATE FILE` was provided
    if let Some(template_file) = &args.template_file {
        // let template_extension = &*args
        let template_extension = &*template_file
            .extension()
            .expect("Template has no file extension.")
            .to_string_lossy();

        let rendered_template_extension = "rendered.".to_string() + template_extension;

        let template_context_file = template_file.with_extension("ctx.json");

        let context_json =
            fs::read_to_string(&template_context_file).expect("Unable to load template context.");

        let context: serde_json::Value =
            serde_json::from_str(&context_json).expect("Unable to parse context JSON.");

        let rendered_output_file = template_file.with_extension(&rendered_template_extension);

        let log_level = match args.verbose {
            1 => LevelFilter::Info,
            2 => LevelFilter::Debug,
            _ => LevelFilter::Error,
        };

        TermLogger::init(
            log_level,
            simplelog::Config::default(),
            simplelog::TerminalMode::Mixed,
            simplelog::ColorChoice::Auto,
        )
        .expect("Unable to initialize logger.");

        log::info!("Rendering File: {}", template_file.to_string_lossy());
        log::info!("Context File: {}", template_context_file.to_string_lossy());

        let template_contents = load_template_file(&template_file);

        let result = match template_contents {
            TemplateKind::Tera(contents) => {
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
            TemplateKind::Handlebars(contents) => {
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
            TemplateKind::Liquid(contents) => {
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
            TemplateKind::Unknown(engine, _) => panic!("Unknown template engine: `{engine}`"),
            TemplateKind::NoEngine(raw) => raw,
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
        let pretty_print_preconditions = [args.pretty, args.verbose > 0];

        if pretty_print_preconditions.iter().any(|&c| c) {
            pretty_print(&result, Some(template_extension))
        } else {
            println!("{result}");
        }

        // TODO: Output to file only if output argument is given
        if let Some(output_path) = args.output_file {
            println!("Activated Output Switch");
            log::info!("Rendered Output File: {}", output_path.to_string_lossy());
            write_to_file(&result, output_path);
            // output_render(&result, rendered_output_file);
        }
    }

    Ok(())
}
