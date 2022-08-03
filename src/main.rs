use anyhow::{anyhow, Context, Result};
use clap::Parser;
use handlebars::Handlebars;
// use human_panic::setup_panic;
use log::LevelFilter;
use regex::RegexBuilder;
use simplelog::TermLogger;
use std::{
    fs::{self, OpenOptions},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};
use tera::Tera;

type Contents = String;
type EngineName = String;

// TODO: 3.8.2022
// DONE: `--open`
// TODO: Updated CLI Descriptions + Description with Source Code and License
// TODO: Build logic for template table ver1
// TODO: Build logic for template table ver2
// TODO: Bonus: `--watch`
// TODO: `--engine`
// TODO: Bonus: `--stdout`

const DEFAULT_CONTEXT_FILE: &str = "default.ctx.json";

enum Template {
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
    #[clap(value_parser, short, long = "context", value_name = "CONTEXT FILE")]
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

    // /Print pretty, highlighted output to the terminal.
    // /
    // /WARNING: CLI output cannot be used for piping as ASCII/UTF-8 is transformed.
    // /Use `--output` switch if you wish to commit the rendered output to file.
    // #[clap(short, long, action)]
    // pretty: bool,

    //
    /// Open rendered output file
    #[clap(short = 'O', long, action)]
    open: bool,

    /// Constantly render changes in template file.  
    #[clap(short, long = "watch", value_name = "SECONDS", action)]
    watch_seconds: Option<u8>,

    /// Print rendered result to STDOUT
    #[clap(short, long, action)]
    stdout: bool,
}

/// Write `content` to file `path` using BufWriter
fn write_to_file<P: AsRef<Path>>(content: &str, path: P) -> Result<()> {
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(&path)
        .with_context(|| {
            format!(
                "Unable to create file: \"{}\"",
                path.as_ref().to_string_lossy()
            )
        })?;

    let mut bw = BufWriter::new(file);
    bw.write_all(content.as_bytes()).with_context(|| {
        format!(
            "Unable to write rendered output to file: \"{}\"",
            path.as_ref().to_string_lossy()
        )
    })?;

    Ok(())
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

impl<'arg> From<TemplateData<'arg>> for Template {
    /// Loads a template file into a Template enum type.
    /// Decides on the engine type by first inspecting the file extension (`.tera`, `.hbs` or `.liq`).
    /// If no special extension is provided then the contents of the template are inspected for the magic comment `<!--TEMPLATE engine_name-->`.
    ///
    /// Engine Names: `tera`, `handlebars` or `hbs`, `liquid` or `liq`
    fn from(td: TemplateData) -> Self {
        // Checking for template file extension to determine the template engine.
        // Notice the early returns.
        if let Some(template_file) = td.file_path {
            if let Some(extension) = template_file.extension() {
                let file_extension = &*extension.to_string_lossy();

                match file_extension {
                    "tera" => return Template::Tera(td.contents),
                    "hbs" => return Template::Handlebars(td.contents),
                    "liq" => return Template::Liquid(td.contents),
                    _ => {} // ignore unknown extensions
                };
            }
        }
        // Scan template contents for the magic comment to return the proper Template kind.
        td.contents.into()
    }
}

// fn pretty_print(content: &str, language: &str) -> Result<()> {
//     let bytes_content = content.as_bytes();

//     PrettyPrinter::new()
//         .language(language) // Default: auto-detect
//         .line_numbers(false)
//         .grid(true)
//         .header(true)
//         .input(bat::Input::from_bytes(bytes_content))
//         .print()
//         .context("Unable to pretty print.")?;

//     Ok(())
// }

fn stdin_read() -> Result<String> {
    let lines = std::io::stdin().lines();
    let mut result = String::new();
    for line in lines {
        let l = line.context("Failed to read STDIN line")?;
        result.push_str(&l);
        result.push('\n');
    }

    // let result = lines
    //     .map(|line| {
    //         let l = line.expect("Failed to read stdin line");
    //         l + "\n"
    //     })
    //     .collect();

    Ok(result)
}

struct TemplateData<'args> {
    contents: String,
    file_path: Option<&'args PathBuf>,
}

// #[allow(unused)]
struct ContextData {
    context: serde_json::Value,
    file_path: PathBuf,
}

struct RenderedTemplate(String);

fn render(template_data: TemplateData, context_data: ContextData) -> Result<RenderedTemplate> {
    // let default_language = "html";

    // let template_language = &*match template_data.file_path {
    //     Some(p) => match p.extension() {
    //         Some(ext) => ext.to_string_lossy(),
    //         None => Cow::Borrowed(default_language),
    //     },
    //     None => Cow::Borrowed(default_language),
    // };

    let template = Template::from(template_data);
    let result = match template {
        Template::Tera(contents) => {
            let context = tera::Context::from_value(context_data.context)
                .context("Tera rejected Context object.")?;

            // match Tera::one_off(&contents, &context, true) {
            //     Ok(rendered) => rendered,
            //     Err(e) => {
            //         if let Some(source) = e.source() {
            //             log::error!("{source}");
            //         }
            //         return Err(anyhow::Error::new(e).context("Unable to render template."));
            //     }
            // }

            Tera::one_off(&contents, &context, true)
                .context("Tera is unable to render the template.")?
        }
        Template::Handlebars(contents) => {
            let handlebars = Handlebars::new();
            let render = handlebars.render_template(&contents, &context_data.context);
            // match render {
            //     Ok(contents) => contents,
            //     Err(e) => {
            //         if let Some(source) = e.source() {
            //             if let Some(template_error) = source.downcast_ref::<TemplateError>() {
            //                 let template_error_string = format!("{template_error}");
            //                 pretty_print(&template_error_string, template_language)?;
            //             }
            //         }
            //         return Err(anyhow::Error::new(e).context("Unable to render template."));
            //     }
            // }
            render.context("Handlebars is unable to render the template.")?
        }
        Template::Liquid(contents) => {
            let template = liquid::ParserBuilder::with_stdlib()
                .build()
                .context("Liquid is unable to build the parser.")?
                .parse(&contents);

            // let template = match template {
            //     Ok(t) => t,
            //     Err(e) => {
            //         let template_error_string = format!("{e}");
            //         pretty_print(&template_error_string, template_language)?;
            //         // eprintln!("{e}");
            //         return Err(anyhow::Error::new(e).context("Unable to parse template."));
            //     }
            // };
            let template = template.context("Liquid is unable to parse the template.")?;

            let globals = liquid::object!(&context_data.context);

            template
                .render(&globals)
                .context("Liquid is unable to render the template.")?
        }
        // Template::Unknown(engine, _) => panic!("Unknown template engine: `{engine}`"),
        Template::Unknown(engine, _) => return Err(anyhow!("Unknown template engine: `{engine}`")),
        Template::NoEngine(raw) => raw,
    };
    Ok(RenderedTemplate(result))
}

fn main() -> Result<()> {
    // setup_panic!();
    let args = Cli::parse();

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
    .context("Unable to initialize the logger.")?;

    let template_file = if let Some(path) = &args.template_file {
        Some(fs::canonicalize(path)?)
    } else {
        None
    };

    let template_data = if let Some(template_file) = &template_file {
        log::info!("Rendering File: \"{}\"", template_file.to_string_lossy());
        TemplateData {
            contents: fs::read_to_string(&template_file).with_context(|| {
                format!(
                    "Unable to load template file \"{}\"",
                    template_file.to_string_lossy()
                )
            })?,
            file_path: Some(template_file),
        }
    } else {
        TemplateData {
            contents: stdin_read()?,
            file_path: None,
        }
    };

    let context_file = if let Some(path) = &args.context_file {
        Some(fs::canonicalize(path)?)
    } else {
        None
    };

    let context_data = {
        let context_file = if let Some(context_file) = &context_file {
            context_file.to_owned()
        } else if let Some(template_file) = &args.template_file {
            let ctx_path = template_file.with_extension("ctx.json");
            if ctx_path.exists() {
                ctx_path
            } else {
                PathBuf::from(DEFAULT_CONTEXT_FILE)
            }
        } else {
            PathBuf::from(DEFAULT_CONTEXT_FILE)
        };

        log::info!("Context File: \"{}\"", context_file.to_string_lossy());

        let contents = fs::read_to_string(&context_file).with_context(|| {
            format!(
                "Unable to load context file \"{}\"",
                context_file.to_string_lossy()
            )
        })?;
        ContextData {
            // context: contents.into(), // not the way to do it as some engines did not recognize the JSON structure.
            context: serde_json::from_str(&contents).with_context(|| {
                format!(
                    "Unable to parse JSON context from file \"{}\"",
                    context_file.to_string_lossy()
                )
            })?,
            file_path: context_file,
        }
    };

    let rendered_template = render(template_data, context_data)?;

    if let Some(output_arg) = args.output_file {
        log::info!("Rendered Output File: \"{}\"", output_arg.to_string_lossy());
        write_to_file(&rendered_template.0, &output_arg)?;
        if args.open {
            opener::open(&output_arg)?;
        }
    } else if let Some(template_file) = args.template_file {
        let mut extension = String::from("rendered");

        if let Some(ext) = template_file.extension() {
            extension.push('.');
            extension.push_str(&*ext.to_string_lossy());
        }

        let mut output_path = template_file.clone();
        output_path.set_extension(extension);

        log::info!(
            "Rendered Output File: \"{}\"",
            output_path.to_string_lossy()
        );
        write_to_file(&rendered_template.0, &output_path)?;
        if args.open {
            opener::open(&output_path)?;
        }
    } else {
        // let pretty_print_preconditions = [args.pretty, args.verbose > 0];
        //     if pretty_print_preconditions.iter().any(|&c| c) {
        //         pretty_print(&result, Some(template_extension))
        //     } else {
        //         println!("{result}");
        //     }
        println!("{}", rendered_template.0);
    }

    // Checks if `<TEMPLATE FILE` was provided
    // if let Some(template_file) = &args.template_file {
    //     // let template_extension = &*args
    //     let template_extension = &*template_file
    //         .extension()
    //         .expect("Template has no file extension.")
    //         .to_string_lossy();

    //     let rendered_template_extension = "rendered.".to_string() + template_extension;

    //     let template_context_file = template_file.with_extension("ctx.json");

    //     let context_json =
    //         fs::read_to_string(&template_context_file).expect("Unable to load template context.");

    //     let context: serde_json::Value =
    //         serde_json::from_str(&context_json).expect("Unable to parse context JSON.");

    //     let rendered_output_file = template_file.with_extension(&rendered_template_extension);

    //     let log_level = match args.verbose {
    //         1 => LevelFilter::Info,
    //         2 => LevelFilter::Debug,
    //         _ => LevelFilter::Error,
    //     };

    //     TermLogger::init(
    //         log_level,
    //         simplelog::Config::default(),
    //         simplelog::TerminalMode::Mixed,
    //         simplelog::ColorChoice::Auto,
    //     )
    //     .expect("Unable to initialize logger.");

    //     log::info!("Rendering File: {}", template_file.to_string_lossy());
    //     log::info!("Context File: {}", template_context_file.to_string_lossy());

    //     let template_contents = load_template_file(&template_file);

    //     let result = match template_contents {
    //         TemplateKind::Tera(contents) => {
    //             let context =
    //                 Context::from_value(context).expect("Unable to create context from JSON.");

    //             match Tera::one_off(&contents, &context, true) {
    //                 Ok(rendered) => rendered,
    //                 Err(e) => {
    //                     if let Some(source) = e.source() {
    //                         log::error!("{source}");
    //                         // eprintln!("{source}");
    //                     }

    //                     panic!("Unable to render template.");
    //                 }
    //             }
    //         }
    //         TemplateKind::Handlebars(contents) => {
    //             let handlebars = Handlebars::new();
    //             let render = handlebars.render_template(&contents, &context);
    //             match render {
    //                 Ok(contents) => contents,
    //                 Err(e) => {
    //                     if let Some(source) = e.source() {
    //                         if let Some(template_error) = source.downcast_ref::<TemplateError>() {
    //                             let template_error_string = format!("{template_error}");
    //                             pretty_print(&template_error_string, Some(template_extension));
    //                             // eprintln!("{template_error}");
    //                         }
    //                     }
    //                     panic!("Unable to render template.");
    //                 }
    //             }
    //         }
    //         TemplateKind::Liquid(contents) => {
    //             let template = liquid::ParserBuilder::with_stdlib()
    //                 .build()
    //                 .expect("Unable to build Liquid parser.")
    //                 .parse(&contents);

    //             let template = match template {
    //                 Ok(t) => t,
    //                 Err(e) => {
    //                     let template_error_string = format!("{e}");
    //                     pretty_print(&template_error_string, Some(template_extension));
    //                     // eprintln!("{e}");
    //                     panic!("Unable to parse template.");
    //                 }
    //             };

    //             let globals = liquid::object!(&context);

    //             template
    //                 .render(&globals)
    //                 .expect("Unable to render template.")
    //         }
    //         TemplateKind::Unknown(engine, _) => panic!("Unknown template engine: `{engine}`"),
    //         TemplateKind::NoEngine(raw) => raw,
    //     };

    //     // Cancelled: PrettyPrint -> Characters are not standard and cannot be redirected properly with pipes.. for now.
    //     // PrettyPrinter::new()
    //     //     .language("html") // Default: auto-detect
    //     //     .line_numbers(true)
    //     //     .grid(true)
    //     //     .header(true)
    //     //     .input(bat::Input::from_bytes(result.as_bytes()))
    //     //     .print()
    //     //     .unwrap();
    //     let pretty_print_preconditions = [args.pretty, args.verbose > 0];

    //     if pretty_print_preconditions.iter().any(|&c| c) {
    //         pretty_print(&result, Some(template_extension))
    //     } else {
    //         println!("{result}");
    //     }

    //     // TODO: Output to file only if output argument is given
    //     if let Some(output_path) = args.output_file {
    //         println!("Activated Output Switch");
    //         log::info!("Rendered Output File: {}", output_path.to_string_lossy());
    //         write_to_file(&result, output_path);
    //         // output_render(&result, rendered_output_file);
    //     }
    // }

    Ok(())
}
