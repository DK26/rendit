use anyhow::{anyhow, Context, Result};
use clap::Parser;
use handlebars::Handlebars;
use path_slash::PathExt;
// use human_panic::setup_panic;
use enum_iterator::{all, Sequence};
use log::LevelFilter;
use regex::RegexBuilder;
use simplelog::TermLogger;
use std::{
    borrow::Borrow,
    env::current_exe,
    ffi::{OsStr, OsString},
    fs::{self, OpenOptions},
    io::{BufWriter, Write},
    ops::Deref,
    path::{Path, PathBuf},
    process,
    str::FromStr,
    thread,
    time::Duration,
};
use tera::Tera;

type Contents = String;
type EngineName = String;

// TODO: 3.8.2022
// TODO: Bonus: Updated CLI Descriptions + Description with Source Code and License
// TODO: Build logic for template table ver1
// TODO: Build logic for template table ver2
// TODO: Bonus: STDIN loop

const DEFAULT_CONTEXT_FILE: &str = "default.ctx.json";
const DEFAULT_WATCH_SLEEP_TIME: Duration = Duration::from_secs(2);

// A simple implementation of `% touch path` (ignores existing files)
// Inspired by: https://doc.rust-lang.org/rust-by-example/std_misc/fs.html
fn touch<P: AsRef<Path>>(path: P) -> Result<()> {
    OpenOptions::new().create(true).write(true).open(path)?;
    Ok(())
}

// This function attempts to be ignorant about any problems.
// It just tries to figure out if a given file path location.
// If the path doesn't exists, it assumes someone else will scream about it.
// On failure, it just returns the original Path.
#[inline]
fn new_canonicalize_path_buf<P: AsRef<Path>>(path: P) -> PathBuf {
    // Canonicalize seem to be having trouble on Windows with relative paths that include a backslash.
    // This work around is meant to make sure that before Canonicalize encounters the given path,
    // its backslashes will be replaced with regular ones so `canonicalize` will be able to handle it.
    let path: PathBuf = if path.as_ref().has_root() {
        path.as_ref().into()
    } else {
        (&*path.as_ref().to_slash_lossy()).into()
    };

    match fs::canonicalize(&path) {
        Ok(abs_path) => abs_path,
        // On failure of getting the full path, keep the relative path.
        //
        // Possible failures of `fs::canonicalize`:
        //  1. path does not exist.
        //  2. A non-final component in path is not a directory.
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => match touch(&path) {
                Ok(_) => new_canonicalize_path_buf(&path),
                Err(_) => path,
            },
            _ => path,
        },
    }
}

// Has the potential to be more correct. For the alpha and beta stages, I'll keep this function around.
#[inline]
fn new_full_path_buf<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
    if path.as_ref().has_root() {
        Ok(path.as_ref().to_owned())
    } else {
        let exe_dir = current_exe()
            .context("Unable to get current executable file location")?
            .parent()
            .context("Unable to get current executable directory")?
            .to_owned();

        Ok(exe_dir.join(path))
    }
}

// TODO: Move to an external crate, improve and with some more ideas and publish on crates.io.
// Should behave just like a `PathBuf` and therefore should have the same methods + New security features (Restrict trait?)
#[derive(Clone, Debug)]
struct AbsolutePath {
    path: PathBuf,
}

impl AbsolutePath {
    #[inline]
    fn into_inner(self) -> PathBuf {
        self.path
    }
}

impl<T: ?Sized + AsRef<OsStr>> From<&T> for AbsolutePath {
    /// Converts a borrowed [`OsStr`] to a [`AbsolutePath`].
    ///
    /// Allocates a [`AbsolutePath`] and copies the data into it.
    #[inline]
    fn from(s: &T) -> AbsolutePath {
        AbsolutePath {
            // path: new_full_path_buf(s.as_ref()).unwrap_or_else(|_| s.into()),
            path: new_canonicalize_path_buf(s.as_ref()),
        }
    }
}

impl From<OsString> for AbsolutePath {
    #[inline]
    fn from(s: OsString) -> Self {
        AbsolutePath {
            // path: new_full_path_buf(&s).unwrap_or_else(|_| s.into()),
            path: new_canonicalize_path_buf(s),
        }
    }
}

impl From<PathBuf> for AbsolutePath {
    #[inline]
    fn from(s: PathBuf) -> Self {
        AbsolutePath {
            // path: new_full_path_buf(&s).unwrap_or(s),
            path: new_canonicalize_path_buf(s),
        }
    }
}

impl FromStr for AbsolutePath {
    type Err = anyhow::Error;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let res = AbsolutePath {
            // path: new_full_path_buf(s).unwrap_or_else(|_| s.into()),
            path: new_canonicalize_path_buf(s),
        };
        Ok(res)
    }
}

impl std::fmt::Display for AbsolutePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path.to_string_lossy())
    }
}

impl AsRef<Path> for AbsolutePath {
    #[inline]
    fn as_ref(&self) -> &Path {
        self.path.as_ref()
    }
}

impl AsRef<PathBuf> for AbsolutePath {
    #[inline]
    fn as_ref(&self) -> &PathBuf {
        &self.path
    }
}

impl AsRef<OsStr> for AbsolutePath {
    #[inline]
    fn as_ref(&self) -> &OsStr {
        self.path.as_ref()
    }
}

impl Borrow<Path> for AbsolutePath {
    #[inline]
    fn borrow(&self) -> &Path {
        self.path.borrow()
    }
}

impl Deref for AbsolutePath {
    type Target = Path;

    #[inline]
    fn deref(&self) -> &Path {
        self.path.deref()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Sequence, strum_macros::Display)]
enum TemplateEngine {
    Tera,
    Liquid,
    Handlebars,
    None,
}

impl FromStr for TemplateEngine {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let res = match s.to_lowercase().as_str() {
            "tera" => TemplateEngine::Tera,
            "liquid" | "liq" => TemplateEngine::Liquid,
            "handlebars" | "hbs" => TemplateEngine::Handlebars,
            "none" => TemplateEngine::None,
            _ => {
                return Err(anyhow!(
                    "Please try one of the supported engines in `--engine-list`"
                ))
            }
        };
        Ok(res)
    }
}

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
    /// The template file path requiring a `default.ctx.json` context file or template specific context file
    /// containing the template name and ending with the `.ctx.json` extension:
    ///
    /// e.g.
    ///  For the Template file `my_template.html`
    ///  the context file would be `my_template.ctx.json`
    ///  When both are located under the same directory.
    ///
    ///  If `my_template.ctx.json` is missing, the tool will attempt to load `default.ctx.json` under the same directory.
    ///
    /// Output:
    ///  - Providing `<TEMPLATE FILE>` file will automatically produce a rendered output file with a proper name and extension: `<TEMPLATE NAME>.rendered.<extension>`.
    ///  - NOT providing `<TEMPLATE FILE>`, will trigger STDIN mode and will attempt to read the template data from STDIN, WITHOUT producing an output file.
    #[clap(value_parser, value_name = "TEMPLATE FILE")]
    template_file: Option<AbsolutePath>,

    /// Override automatic loading of the context file with the specified context file.
    #[clap(value_parser, short, long = "context", value_name = "CONTEXT FILE")]
    context_file: Option<AbsolutePath>,

    /// Override automatic output file path with the specified file path.
    #[clap(value_parser, short, long = "output", value_name = "OUTPUT FILE")]
    output_file: Option<AbsolutePath>,

    /// Sets the level of verbosity.
    ///  
    /// `-v` sets logging level to INFO
    /// `-vv` sets logging level to DEBUG
    ///
    /// WARNING: Effects CLI / STDOUT output.
    /// Use the `--output` switch if you wish to commit the rendered output to file.
    /// Use the `--stderr` switch to avoid including the logger messages in the final output.
    #[clap(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    // /Print pretty, highlighted output to the terminal.
    // /
    // /WARNING: CLI output cannot be used for piping as ASCII/UTF-8 is transformed.
    // /Use `--output` switch if you wish to commit the rendered output to file.
    // #[clap(short, long, action)]
    // pretty: bool,

    //
    /// Open the rendered output file with a default software.
    #[clap(short = 'O', long, action)]
    open: bool,

    /// Constantly render changes of both the template and the context files for every 2 seconds.  
    #[clap(short, long, action)]
    watch: bool,

    /// Print rendered result to STDOUT.
    #[clap(short, long, action)]
    stdout: bool,

    /// Print rendered result to STDERR.
    #[clap(short, long, action)]
    stderr: bool,

    /// Force rendering with the specified render engine.
    /// Use only when there is no magic comment or a template file extension available.
    #[clap(value_parser, short = 'e', long = "engine", value_name = "ENGINE NAME")]
    engine: Option<TemplateEngine>,

    /// Print supported engine list for the `--engine` option.
    #[clap(long, action)]
    engine_list: bool,
}

/// Write `content` to file `path` using BufWriter
fn write_to_file<P: AsRef<Path>>(content: &str, path: P) -> Result<()> {
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
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
    file_path: Option<&'args AbsolutePath>,
}

// #[allow(unused)]
struct ContextData {
    context: serde_json::Value,
    file_path: AbsolutePath,
}

struct RenderedTemplate(String);

enum EngineDetection {
    Auto,
    Force(TemplateEngine),
}

impl From<Option<TemplateEngine>> for EngineDetection {
    fn from(te: Option<TemplateEngine>) -> Self {
        match te {
            Some(engine) => EngineDetection::Force(engine),
            None => EngineDetection::Auto,
        }
    }
}

fn render(
    template_data: TemplateData,
    context_data: ContextData,
    engine_detection: EngineDetection,
) -> Result<RenderedTemplate> {
    // let default_language = "html";

    // let template_language = &*match template_data.file_path {
    //     Some(p) => match p.extension() {
    //         Some(ext) => ext.to_string_lossy(),
    //         None => Cow::Borrowed(default_language),
    //     },
    //     None => Cow::Borrowed(default_language),
    // };

    let template = match engine_detection {
        EngineDetection::Auto => Template::from(template_data),
        EngineDetection::Force(engine) => {
            log::debug!("Forced Engine: `{engine}`");
            match engine {
                TemplateEngine::Tera => Template::Tera(template_data.contents),
                TemplateEngine::Liquid => Template::Liquid(template_data.contents),
                TemplateEngine::Handlebars => Template::Handlebars(template_data.contents),
                TemplateEngine::None => Template::NoEngine(template_data.contents),
            }
        }
    };

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

    if args.engine_list {
        let supported_engines = all::<TemplateEngine>().collect::<Vec<_>>();

        for (i, engine) in supported_engines.iter().enumerate() {
            println!("{}. {}", i + 1, engine.to_string().as_str().to_lowercase());
        }
        process::exit(0);
    }

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

    let mut has_looped = false;

    'main: loop {
        let template_file = &args.template_file;

        let template_data = if let Some(template_file) = &template_file {
            log::info!("Rendering File: \"{template_file}\"");
            TemplateData {
                contents: fs::read_to_string(&template_file)
                    .with_context(|| format!("Unable to load template file \"{template_file}\""))?,
                file_path: Some(template_file),
            }
        } else {
            TemplateData {
                contents: stdin_read()?,
                file_path: None,
            }
        };

        let context_file = &args.context_file;
        // let context_file = if let Some(path) = &args.context_file {
        //     Some(path.canonicalize()?)
        // } else {
        //     None
        // };

        let context_data = {
            let context_file = if let Some(context_file) = &context_file {
                context_file.to_owned()
            } else if let Some(template_file) = &args.template_file {
                let ctx_path = template_file.with_extension("ctx.json");

                if ctx_path.exists() {
                    ctx_path.into()
                } else {
                    PathBuf::from(DEFAULT_CONTEXT_FILE).into()
                }
            } else {
                PathBuf::from(DEFAULT_CONTEXT_FILE).into()
            };

            log::info!("Context File: \"{context_file}\"");

            let contents = fs::read_to_string(&context_file)
                .with_context(|| format!("Unable to load context file \"{context_file}\""))?;
            ContextData {
                // context: contents.into(), // not the way to do it as some engines did not recognize the JSON structure.
                context: serde_json::from_str(&contents).with_context(|| {
                    format!("Unable to parse JSON context from file \"{context_file}\"")
                })?,
                file_path: context_file,
            }
        };

        let rendered_template = render(template_data, context_data, args.engine.into())?;

        if args.stderr {
            eprintln!("{}", rendered_template.0);
        }

        if args.stdout {
            println!("{}", rendered_template.0);
        }

        // Output stages
        if let Some(ref output_arg) = args.output_file {
            log::info!("Rendered Output File: \"{output_arg}\"");
            write_to_file(&rendered_template.0, &output_arg)?;
            if !has_looped && args.open {
                log::info!("Opening: \"{output_arg}\"");
                opener::open(&output_arg)?;
            }
        } else if let Some(ref template_file) = args.template_file {
            let mut extension = String::from("rendered");

            if let Some(ext) = template_file.extension() {
                extension.push('.');
                extension.push_str(&*ext.to_string_lossy());
            }

            let mut output_path = template_file.to_path_buf();
            output_path.set_extension(extension);

            let output_path: AbsolutePath = output_path.into();

            log::info!("Rendered Output File: \"{output_path}\"");
            write_to_file(&rendered_template.0, &output_path)?;
            if !has_looped && args.open {
                log::info!("Opening: \"{output_path}\"");
                opener::open(&output_path)?;
            }
        } else if !args.stdout {
            // let pretty_print_preconditions = [args.pretty, args.verbose > 0];
            //     if pretty_print_preconditions.iter().any(|&c| c) {
            //         pretty_print(&result, Some(template_extension))
            //     } else {
            //         println!("{result}");
            //     }
            println!("{}", rendered_template.0);
        }
        if args.watch {
            thread::sleep(DEFAULT_WATCH_SLEEP_TIME);
            has_looped = true;
        } else {
            break 'main;
        }
    }
    Ok(())
}
