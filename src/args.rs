use std::path::PathBuf;

#[derive(argh::FromArgs)]
/// An interpreter for a simple instruction language that I made for fun.
pub struct Args {
    #[argh(positional)]
    /// the path to the program to run
    pub path: PathBuf,
}
