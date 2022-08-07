use ::std::marker::PhantomData;
use anyhow::{anyhow, Context, Result};
use clap::ArgMatches;

pub trait ApplicationState {}

pub struct Application<S: ApplicationState> {
    arg_matches: ArgMatches,
    state: PhantomData<S>,
}

// App Modes to help separate logical flows
struct FileMode;
impl ApplicationState for FileMode {}
struct StdinMode;
impl ApplicationState for StdinMode {}

trait Run {
    fn run() -> Result<()>;
}

impl Run for Application<FileMode> {
    fn run() -> Result<()> {
        'watch: loop {
            todo!();
        }
        Ok(())
    }
}

impl Run for Application<StdinMode> {
    fn run() -> Result<()> {
        'main: loop {
            todo!();
            // break 'main;
        }
        Ok(())
    }
}
