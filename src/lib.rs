//! The core library to convert ELF to PKE
pub mod logger;
use clap::Parser;
use std::path::PathBuf;

/// The struct which contains the arguments.
#[derive(Parser)]
pub struct Args {
    /// Whether the output file will assign to driver. Default: false.
    #[arg(short = 'd', long = "driver", required = false)]
    pub is_driver: bool,

    /// Whether the unloadable is being stripped. Default: false.
    #[arg(short = 's', long = "strip", required = false)]
    pub strip_unloadable: bool,

    /// Specify the output dir. Default: `out.pke`.
    #[arg(short = 'o', long = "output")]
    pub output_path: Option<PathBuf>,

    /// Specify the app name. Default: `appname`.
    #[arg(short = 'n', long = "name")]
    pub appname: Option<String>,

    /// Specify the author name. Default: `example`.
    #[arg(short = 'a', long = "author")]
    pub author: Option<String>,

    /// Specify the path of the input file.
    #[arg(required = true)]
    pub path: PathBuf,
}

/// The struct which contains the sections.
#[derive(Debug, Clone)]
pub struct Sections<'a> {
    /// The section name. Max for 16 bytes.
    pub section_name: &'a str,

    /// Assign is this loadable.
    pub is_loadable: bool,

    /// Assign is this executable.
    pub is_executable: bool,

    /// The section virtual start address.
    pub vaddr: u64,

    /// The section size.
    pub size: u64,

    /// The slice which point to the content.
    pub data: &'a [u8],
}