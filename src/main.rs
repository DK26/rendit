use anyhow::{anyhow, Context, Result};
use clap::{value_parser, Arg};
use handlebars::Handlebars;
use path_slash::PathExt;
// use human_panic::setup_panic;
use enum_iterator::{all, Sequence};
use log::LevelFilter;
use regex::{Regex, RegexBuilder};
use simplelog::TermLogger;
use std::{
    borrow::{Borrow, Cow},
    ffi::{OsStr, OsString},
    fs::{self, OpenOptions},
    io::{BufWriter, Write},
    ops::Deref,
    path::{Path, PathBuf},
    process,
    rc::Rc,
    str::FromStr,
    thread,
    time::Duration,
};
use tera::Tera;

type Contents = Rc<String>;
type EngineName = String;

// TODO: 9.8.2022
// TODO: Bonus: STDIN loop
// TODO: Enable usage of external templates for `include` [Liquid](https://github.com/leftwm/leftwm/issues/439)
// TODO: Logic for printing errors within `--watch`, should apply only when `.ctx.json` or template are changing. Maybe we should loop just when changes are detected while it checks for changes every 2 seconds by default.

const DEFAULT_CONTEXT_FILE: &str = "default.ctx.json";

/// Scan the template for reference to other templates, such as:
/// `{% include %}`, `{% extend %}` or `{% import %}` calls
#[inline]
fn find_template_references<P: AsRef<Path>>(content: &str, cwd: Option<P>) -> Vec<AbsolutePath> {
    let re = Regex::new(
        r#"\{%\s+?(?:import|include|extend)\s+?"(?P<template>[a-zA-Z0-9.\-/\\_]+?)"\s.*?%\}"#,
    )
    .expect("Bad regex pattern.");

    let mut buf: Vec<AbsolutePath> = Vec::new();

    log::debug!("Scanning for template references...");

    for cap in re.captures_iter(content) {
        log::debug!("Detected reference: \"{}\"", &cap["template"]);
        // TODO: Make path relative to main template
        let path = if let Some(p) = &cwd {
            p.as_ref().with_file_name(&cap["template"]).into()
        } else {
            cap["template"].into()
        };

        buf.push(path);
    }
    buf
}

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
                Ok(_) => {
                    let res = new_canonicalize_path_buf(&path);
                    match fs::remove_file(&res) {
                        Ok(_) => {
                            log::debug!(
                                "canonicalize(): Removed touched file: \"{}\"",
                                res.to_string_lossy()
                            )
                        }
                        Err(_) => {
                            log::error!(
                                "canonicalize(): Unable to remove file after touch: \"{}\"",
                                res.to_string_lossy()
                            )
                        }
                    };
                    res
                }
                Err(_) => path,
            },
            _ => path,
        },
    }
}

// Has the potential to be more correct. For the alpha and beta stages, I'll keep this function around.
// #[inline]
// fn new_full_path_buf<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
//     if path.as_ref().has_root() {
//         Ok(path.as_ref().to_owned())
//     } else {
//         let exe_dir = current_exe()
//             .context("Unable to get current executable file location")?
//             .parent()
//             .context("Unable to get current executable directory")?
//             .to_owned();

//         Ok(exe_dir.join(path))
//     }
// }

// TODO: Move to an external crate, improve and with some more ideas and publish on crates.io.
// TODO: `AbsolutePath` features should be implemented on `PathBuf` directly with proper traits, to avoid duplicating and interswitching between the types, making it seamless.
// Old Note: Should behave just like a `PathBuf` and therefore should have the same methods + New security features (Restrict trait?)
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

impl AsRef<Path> for AbsolutePath {
    #[inline]
    fn as_ref(&self) -> &Path {
        self.path.as_ref()
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

// impl AsRef<Path> for &AbsolutePath {
//     #[inline]
//     fn as_ref(&self) -> &Path {
//         self.path.as_ref()
//     }
// }

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

/// Supported template engines
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

impl Template {
    fn get_engine(&self) -> &'static str {
        match self {
            Template::Tera(_) => "tera",
            Template::Handlebars(_) => "handlebars",
            Template::Liquid(_) => "liquid",
            Template::Unknown(_, _) => "unknown",
            Template::NoEngine(_) => "no_engine",
        }
    }
}

struct Args {
    template_file: Option<AbsolutePath>,
    context_file: Option<AbsolutePath>,
    output_file: Option<AbsolutePath>,
    verbose: u8,
    open: bool,
    watch: Option<u64>,
    stdout: bool,
    stderr: bool,
    engine: Option<TemplateEngine>,
    engine_list: bool,
    extension: Option<String>,
}

impl Args {
    fn parse() -> Self {
        let arg_matches = {
            let about = format!(
                "{description}\n\n  Author: {author}\n  Source: {source}\n  License: {license}",
                description = env!("CARGO_PKG_DESCRIPTION"),
                author = env!("CARGO_PKG_AUTHORS"),
                source = env!("CARGO_PKG_REPOSITORY"),
                license = env!("CARGO_PKG_LICENSE")
            );

            clap::Command::new(env!("CARGO_PKG_NAME"))
            .version(env!("CARGO_PKG_VERSION"))
            .about(about.as_str())
            .arg(
                Arg::new("template_file")
                    .value_name("TEMPLATE FILE")
                    .long_help(
r#"The template file to render.
    
Unless using the `--context <CONTEXT FILE>`option, this requires either the 
`<TEMPLATE NAME>.ctx.json` or the `default.ctx.json` context files to be present in the template's directory.

[Example]

.
├── my_template.ctx.json
└── my_template.html

If `my_template.ctx.json` is missing, `default.ctx.json` will be loaded automatically from the template's directory.

.
├── default.ctx.json
└── my_template.html

If `default.ctx.json` is missing from the template directory, it will be loaded automatically from the current working directory, instead.

[Output]
    
Unless using the `--output <OUTPUT FILE>` option, providing `<TEMPLATE FILE>`, produces a `<TEMPLATE NAME>.rendered.<EXTENSION>` file by default.

.
├── my_template.html
└── my_template.rendered.html
                   
By NOT providing `<TEMPLATE FILE>`, STDIN mode is activated. Template data stream will be expected to come from STDIN, printing the results to STDOUT instead of writing to file."#
                )
                    .value_parser(value_parser!(AbsolutePath))
                    .display_order(1)
            ).arg(
                Arg::new("context_file")
                    .value_name("CONTEXT FILE")
                    .long_help("Override default loading of the context file with the specified context file.")
                    .short('c')
                    .long("context")
                    .value_parser(value_parser!(AbsolutePath))
                    .display_order(2)
            ).arg(
                Arg::new("output_file")
                    .value_name("OUTPUT FILE")
                    .long_help("Override default output file path with the specified file path.")
                    .short('o')
                    .long("output")
                    .value_parser(value_parser!(AbsolutePath))
                    .display_order(3)
            ).arg(
                Arg::new("stdout")
                    .long_help("Print rendered result to STDOUT.")
                    .long("stdout")
                    .action(clap::ArgAction::SetTrue)
                    .display_order(4)
            ).arg(
                Arg::new("stderr")
                    .long_help("Print rendered result to STDERR.")
                    .long("stderr")
                    .action(clap::ArgAction::SetTrue)
                    .display_order(5)
            ).arg(
                Arg::new("open")
                    .long_help("Open the rendered output file with a default software.")
                    .long("open")
                    .short('O')
                    .action(clap::ArgAction::SetTrue)
                    .display_order(6)
            ).arg(
                Arg::new("watch")
                    .long_help("Constantly render changes in the template with the context file every 2 seconds by default.")
                    .long("watch")
                    .short('w')
                    .value_name("SECONDS")
                    .default_missing_value("2")
                    .value_parser(value_parser!(u64))
                    .display_order(7)
            ).arg(
                Arg::new("engine")
                    .value_name("ENGINE NAME")
                    .long_help(
r#"Force rendering with the specified render engine.
Use only when there is no magic comment or a template file extension available."#
                )
                    .long("engine")
                    .short('e')
                    .value_parser(value_parser!(TemplateEngine))
                    .display_order(8)
            ).arg(
                Arg::new("engine_list")
                    .long_help("Print supported engine list for the `--engine <ENGINE NAME>` option.")
                    .long("engine-list")
                    .action(clap::ArgAction::SetTrue)
                    .display_order(9)
            ).arg(
                Arg::new("extension")
                    .value_name("EXTENSION")
                    .long_help("Force the underlying template engine to treat the template data as if it is a file of the given extension. This may affect escaping of special characters.")
                    .long("extension")
                    .short('x')
                    .value_parser(value_parser!(String))
                    .display_order(10)
            ).arg(
                Arg::new("verbose")
                    .long_help(
r#"Set the level of verbosity.
    
`-v` sets logging level to INFO
    
`-vv` sets logging level to DEBUG
    
`-vvv` sets logging level to TRACE
    
WARNING: Effects CLI / STDOUT output.
Use the `--output` switch if you wish to commit the rendered output to file.
Use the `--stderr` switch to avoid including the logger messages in the final output."#
                )
                .long("verbose")
                .short('v')
                .action(clap::ArgAction::Count)
                .value_parser(value_parser!(u8))
                .display_order(11)
            )
            .get_matches()
        };

        let err_msg = "Bad argument configuration";

        Args {
            template_file: arg_matches
                .get_one::<AbsolutePath>("template_file")
                .map(|v| v.to_owned()),

            context_file: arg_matches
                .get_one::<AbsolutePath>("context_file")
                .map(|v| v.to_owned()),

            output_file: arg_matches
                .get_one::<AbsolutePath>("output_file")
                .map(|v| v.to_owned()),

            verbose: *arg_matches.get_one::<u8>("verbose").expect(err_msg),

            open: *arg_matches.get_one::<bool>("open").expect(err_msg),

            watch: arg_matches.get_one::<u64>("watch").map(|v| v.to_owned()),

            stdout: *arg_matches.get_one::<bool>("stdout").expect(err_msg),

            stderr: *arg_matches.get_one::<bool>("stderr").expect(err_msg),

            engine: arg_matches
                .get_one::<TemplateEngine>("engine")
                .map(|v| v.to_owned()),

            engine_list: *arg_matches.get_one::<bool>("engine_list").expect(err_msg),

            extension: arg_matches
                .get_one::<String>("extension")
                .map(|v| v.to_owned()),
        }
    }
}

// impl<'arg_matches> Args<'arg_matches> {
//     fn parse(arg_matches: &ArgMatches) -> Args {
//         // let arg_matches = arg_matches();

//         let err_msg = "Bad argument configuration";

//         Args {
//             template_file: arg_matches.get_one::<AbsolutePath>("template_file"),
//             context_file: arg_matches.get_one::<AbsolutePath>("context_file"),
//             output_file: arg_matches.get_one::<AbsolutePath>("output_file"),
//             verbose: *arg_matches.get_one::<u8>("verbose").expect(err_msg),
//             open: *arg_matches.get_one::<bool>("open").expect(err_msg),
//             watch: arg_matches.get_one::<u64>("watch"),
//             stdout: *arg_matches.get_one::<bool>("stdout").expect(err_msg),
//             stderr: *arg_matches.get_one::<bool>("stderr").expect(err_msg),
//             engine: arg_matches.get_one::<TemplateEngine>("engine"),
//             engine_list: *arg_matches.get_one::<bool>("engine_list").expect(err_msg),
//             extension: arg_matches.get_one::<String>("extension"),
//         }
//     }
// }

// FIXME: Using `.html.tera` extension, doesn't produce `.html` rendered output
// struct TemplateFile<'path> {
//     path: Rc<PathBuf>,
//     parts: TemplateParts<'path>,
// }
// struct TemplateParts<'path> {
//     name: Cow<'path, str>,
//     extension: Option<&'path OsStr>,
//     kind: Option<TemplateEngine>,
// }
// impl<'path> From<PathBuf> for TemplateFile<'path> {
//     fn from(path: PathBuf) -> Self {
//         let path = Rc::new(path);
//         let s =
//         TemplateFile {
//             path: path.clone(),
//             parts: TemplateParts {
//                 name: path.to_string_lossy(),
//                 extension: path.as_path().extension(),
//                 kind: (),
//             },
//         }
//     }
// }

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

impl From<&str> for Template {
    /// Inspect the String contents for a magic comment `<!--template engine_name-->`, and return the appropriate `Template` enum variation for rendering.
    fn from(contents: &str) -> Self {
        let re = RegexBuilder::new(r#"<!--template\s+(?P<engine>\w+)\s?-->"#)
            .case_insensitive(true)
            .build()
            .expect("Bad regex pattern.");

        let mut re_caps = re.captures_iter(contents);

        // We want to find only the first one without scanning the rest of the file
        let mut re_iter = re.find_iter(contents);

        if let Some(m) = re_iter.next() {
            let found_match = m.as_str();

            let contents = Rc::new(contents.replacen(found_match, "", 1).trim().to_owned());

            let cap = re_caps
                .next()
                .expect("Match without a capture? how is it possible?");

            let engine = cap["engine"].to_lowercase();

            log::debug!("Detected magic comment: `{engine}`");

            match engine.as_str() {
                "tera" => Template::Tera(contents),
                "hbs" | "handlebars" => Template::Handlebars(contents),
                "liq" | "liquid" => Template::Liquid(contents),
                unknown_engine => Template::Unknown(unknown_engine.to_owned(), contents),
            }
        } else {
            Template::NoEngine(Rc::new(contents.to_owned()))
        }
    }
}

impl<'arg> From<&TemplateData<'arg>> for Template {
    /// Loads a template file into a Template enum type.
    /// Decides on the engine type by first inspecting the file extension (`.tera`, `.hbs` or `.liq`).
    /// If no special extension is provided then the contents of the template are inspected for the magic comment `<!--TEMPLATE engine_name-->`.
    ///
    /// Engine Names: `tera`, `handlebars` or `hbs`, `liquid` or `liq`
    fn from(td: &TemplateData) -> Self {
        // Checking for template file extension to determine the template engine.
        // Notice the early returns.
        if let Some(ref template_file) = td.file_path {
            // if let Some(extension) = template_file.extension() {
            if let Some(ref extension) = template_file.parts.extension {
                // let file_extension = &*extension.to_string_lossy();
                let file_extension = extension.as_str();

                // FIXME: Using `.html.tera` extension, doesn't produce `.html` rendered output
                let contents = td.contents.clone();
                match file_extension {
                    "tera" => return Template::Tera(contents),
                    "hbs" => return Template::Handlebars(contents),
                    "liq" => return Template::Liquid(contents),
                    _ => {} // ignore unknown extensions
                };
            }
        }
        // Scan template contents for the magic comment to return the proper Template kind.
        (*td.contents.as_str()).into()
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
        result.push('\r');
        result.push('\n');
    }

    // let result = lines
    //     .map(|line| {
    //         let l = line.expect("Failed to read stdin line");
    //         l + "\r\n"
    //     })
    //     .collect();

    Ok(result)
}

#[derive(thiserror::Error, Debug)]
pub enum RenditError {
    #[error("Path must not be empty or root.")]
    IllegalPath,
}

struct TemplateFile<'path> {
    path: Rc<&'path Path>,
    parts: FileParts,
}

impl<'path> TryFrom<&'path Path> for TemplateFile<'path> {
    type Error = RenditError;

    fn try_from(path: &'path Path) -> Result<Self, Self::Error> {
        let path = Rc::new(path);
        let file = TemplateFile {
            path: path.clone(),
            parts: FileParts::try_from(*path)?,
        };
        Ok(file)
    }
}

struct FileParts {
    name: String,
    kind: Option<String>,
    extension: Option<String>,
}

impl TryFrom<&Path> for FileParts {
    type Error = RenditError;

    fn try_from(pb: &Path) -> Result<Self, Self::Error> {
        let path_string = pb
            .file_name()
            .ok_or(RenditError::IllegalPath)?
            .to_string_lossy();
        let parts: Vec<&str> = path_string.split_terminator('.').collect();
        let res = match parts.len() {
            3 => FileParts {
                name: parts[0].to_owned(),
                kind: Some(parts[1].to_owned()),
                extension: Some(parts[2].to_owned()),
            },
            2 => FileParts {
                name: parts[0].to_owned(),
                kind: None,
                extension: Some(parts[1].to_owned()),
            },
            1 => FileParts {
                name: parts[0].to_owned(),
                kind: None,
                extension: None,
            },
            _ => return Err(RenditError::IllegalPath),
        };
        Ok(res)
    }
}

struct TemplateData<'a> {
    contents: Rc<String>,
    // file_path: Option<&'a AbsolutePath>,
    file_path: Option<TemplateFile<'a>>,
}

// #[allow(unused)]
struct ContextData {
    context: serde_json::Value,
    file_path: AbsolutePath,
}

struct RenderedTemplate(Rc<String>);
// struct RenderedTemplate(String);
// enum RenderResult<'contents> {
//     Processed(RenderedTemplate),
//     Raw(&'contents str),
// }

enum DetectionMethod {
    Auto,
    Force(TemplateEngine),
}

impl From<Option<TemplateEngine>> for DetectionMethod {
    fn from(te: Option<TemplateEngine>) -> Self {
        match te {
            Some(engine) => DetectionMethod::Force(engine),
            None => DetectionMethod::Auto,
        }
    }
}

impl From<Option<&TemplateEngine>> for DetectionMethod {
    fn from(te: Option<&TemplateEngine>) -> Self {
        match te {
            Some(engine) => DetectionMethod::Force(*engine),
            None => DetectionMethod::Auto,
        }
    }
}

enum TemplateExtension<'a> {
    Auto,
    Force(&'a str),
}

impl<'a> From<Option<&'a String>> for TemplateExtension<'a> {
    fn from(s: Option<&'a String>) -> Self {
        match s {
            Some(ext) => TemplateExtension::Force(ext),
            None => TemplateExtension::Auto,
        }
    }
}

fn render<'a>(
    template_data: &'a TemplateData,
    context_data: &'a ContextData,
    engine_detection: DetectionMethod,
    template_extension: TemplateExtension,
) -> Result<RenderedTemplate> {
    // ) -> Result<RenderedTemplate<'a>> {
    // let default_language = "html";

    // let template_language = &*match template_data.file_path {
    //     Some(p) => match p.extension() {
    //         Some(ext) => ext.to_string_lossy(),
    //         None => Cow::Borrowed(default_language),
    //     },
    //     None => Cow::Borrowed(default_language),
    // };

    // let template_path = template_data.file_path.clone();

    let template = match engine_detection {
        DetectionMethod::Auto => {
            log::debug!("Detection method: Automatic");
            Template::from(template_data)
        }
        DetectionMethod::Force(engine) => {
            log::debug!("Detection method: Manual = `{engine}`");
            let contents = template_data.contents.clone();
            match engine {
                TemplateEngine::Tera => Template::Tera(contents),
                TemplateEngine::Liquid => Template::Liquid(contents),
                TemplateEngine::Handlebars => Template::Handlebars(contents),
                TemplateEngine::None => Template::NoEngine(contents),
            }
        }
    };

    log::debug!("Selected engine: `{}`", template.get_engine());

    let result = match template {
        Template::Tera(contents) => {
            let context = tera::Context::from_value(context_data.context.clone())
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

            let templates_root_file = if let Some(ref template_file) = template_data.file_path {
                Cow::Borrowed(*template_file.path)
            } else {
                let abs_path: AbsolutePath = std::env::current_exe()
                    .context("Failed to get current exe path")?
                    .into();
                Cow::Owned(abs_path.into_inner())
            };

            let templates_home_dir = templates_root_file
                .parent()
                .context("Failed to get home directory")?;

            let templates_home_dir_glob = templates_home_dir.join("**");

            let templates_home_dir_glob = templates_home_dir_glob.join("*.*");

            let templates_home_dir_glob = templates_home_dir_glob.to_string_lossy();

            log::debug!("Tera templates path: {templates_home_dir_glob}");

            // TODO: Better to create an instance of `Tera::default()` and have a deep scan for the templates to add only the references ones into a HashSet, than to add every file in the template's directory.
            // let mut tera = Tera::default();

            // let template_references: Vec<(AbsolutePath, Option<String>)> =
            //     find_template_references(&contents, template_path)
            //         .into_iter()
            //         .map(|p| {
            //             let file_name = p.file_name().map(|fp| fp.to_string_lossy().to_string());
            //             (p, file_name)
            //         })
            //         .collect();

            // tera.add_template_files(template_references)
            //     .context("Tera failed loading partial template files")?;

            let mut tera =
                Tera::new(&templates_home_dir_glob).context("Unable to create Tera instance")?;

            // Force extension or auto detect (default `.html`)
            let template_type = if let TemplateExtension::Force(ext) = template_extension {
                log::debug!("Tera: Forcing extension \"{ext}\"");
                Cow::Borrowed(ext)
            } else if let Some(ref path) = template_data.file_path {
                // match path.extension() {
                //     Some(ext) => ext.to_string_lossy(),
                //     None => Cow::Borrowed("html"),
                // }
                match path.parts.extension {
                    Some(ref ext) => Cow::Borrowed(ext.as_str()),
                    None => Cow::Borrowed("html"),
                }
            } else {
                Cow::Borrowed("html")
            };

            log::debug!("Tera: Using extension \"{template_type}\"");
            let in_memory_template = format!("__in_memory__.{}", template_type);

            // Adds a virtual in-memory file for the main template. We need the `.html` extension to enforce HTML escaping.
            tera.add_raw_template(&in_memory_template, &contents)
                .context("Tera is unable to add the main template as raw template.")?;

            let rendered = tera
                .render(&in_memory_template, &context)
                .context("Tera is unable to render the template.")?;

            Rc::new(rendered)
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
            let rendered = render.context("Handlebars is unable to render the template.")?;

            Rc::new(rendered)
        }
        Template::Liquid(contents) => {
            // TODO: Enable partials using `find_template_references()`
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

            let rendered = template
                .render(&globals)
                .context("Liquid is unable to render the template.")?;

            Rc::new(rendered)
        }
        Template::Unknown(engine, _) => return Err(anyhow!("Unknown template engine: `{engine}`")),
        Template::NoEngine(raw) => raw,
    };
    Ok(RenderedTemplate(result))
}

fn main() -> Result<()> {
    // setup_panic!();
    let args = Args::parse();

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
        3 => LevelFilter::Trace,
        _ => LevelFilter::Off,
    };

    TermLogger::init(
        log_level,
        simplelog::Config::default(),
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )
    .context("Unable to initialize the logger.")?;

    let mut has_looped = false;

    let mut last_error: Option<anyhow::Error> = None;

    'watch: loop {
        let template_file_arg = args.template_file.as_ref();

        let template_data = if let Some(template_file) = template_file_arg {
            log::info!("Rendering file: \"{template_file}\"");

            // File Mode
            TemplateData {
                contents: {
                    let contents = fs::read_to_string(&template_file).with_context(|| {
                        format!("Unable to load template file \"{template_file}\"")
                    })?;
                    Rc::new(contents)
                },
                file_path: {
                    let path: &Path = template_file.as_ref();
                    Some(path.try_into()?)
                },
            }
        } else {
            // STDIN Mode
            TemplateData {
                contents: Rc::new(stdin_read()?),
                file_path: None,
            }
        };

        let context_file_arg = args.context_file.as_ref();

        let context_data = {
            let context_file = if let Some(context_file) = context_file_arg {
                context_file.to_owned()
            } else if let Some(template_file) = template_file_arg {
                let ctx_path = template_file.with_extension("ctx.json");

                if ctx_path.exists() {
                    ctx_path.into()
                } else {
                    let relative_default_ctx_file =
                        template_file.with_file_name(DEFAULT_CONTEXT_FILE);
                    if relative_default_ctx_file.exists() {
                        relative_default_ctx_file.into()
                    } else {
                        // Default context from Working directory
                        PathBuf::from(DEFAULT_CONTEXT_FILE).into()
                    }
                }
            } else {
                // Default context from Working directory
                PathBuf::from(DEFAULT_CONTEXT_FILE).into()
            };

            log::info!("Context file: \"{context_file}\"");

            let contents = fs::read_to_string(&context_file)
                .with_context(|| format!("Unable to load context file \"{context_file}\""))?;

            ContextData {
                // context: contents.into(), // not the way to do it as some engines did not recognize the JSON structure.
                context: match serde_json::from_str(&contents).with_context(|| {
                    format!("Unable to parse JSON context from file \"{context_file}\"")
                }) {
                    Ok(ctx) => ctx,
                    Err(e) => match args.watch {
                        Some(secs) => {
                            if let Some(ref le) = last_error {
                                if format!("{le:#}") == format!("{e:#}") {
                                    thread::sleep(Duration::from_secs(secs));
                                    continue 'watch;
                                }
                            };
                            eprintln!("{:?}", e);
                            last_error = Some(e);

                            thread::sleep(Duration::from_secs(secs));
                            continue 'watch;
                        }
                        None => return Err(e),
                    },
                },
                file_path: context_file,
            }
        };

        let rendered_template = match render(
            &template_data,
            &context_data,
            args.engine.into(),
            args.extension.as_ref().into(),
        ) {
            Ok(r) => r,
            Err(e) => match args.watch {
                Some(secs) => {
                    if let Some(ref le) = last_error {
                        if format!("{le:#}") == format!("{e:#}") {
                            thread::sleep(Duration::from_secs(secs));
                            continue 'watch;
                        }
                    };
                    eprintln!("{:?}", e);
                    last_error = Some(e);

                    thread::sleep(Duration::from_secs(secs));
                    continue 'watch;
                }

                None => return Err(e),
            },
        };

        if args.stderr {
            eprintln!("{}", rendered_template.0);
        }

        if args.stdout {
            println!("{}", rendered_template.0);
        }

        // Output stages
        if let Some(ref output_arg) = args.output_file {
            log::info!("Rendered output file: \"{output_arg}\"");
            write_to_file(&rendered_template.0, &output_arg)?;

            if !has_looped && args.open {
                log::info!("Opening: \"{output_arg}\"");
                opener::open(&output_arg)?;
            }
        } else if let Some(template_file) = template_file_arg {
            let mut extension = String::from("rendered");

            if let Some(ext) = template_file.extension() {
                extension.push('.');
                extension.push_str(&*ext.to_string_lossy());
            }

            let mut output_path = template_file.to_path_buf();
            output_path.set_extension(extension);

            let output_path: AbsolutePath = output_path.into();

            log::info!("Rendered output file: \"{output_path}\"");
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
        if let Some(sleep_time) = args.watch {
            log::debug!("Watch mode is activated: Rendering every {sleep_time} seconds");
            thread::sleep(Duration::from_secs(sleep_time));
            has_looped = true;
        } else {
            break 'watch;
        }
    }
    Ok(())
}
