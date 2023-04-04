use anyhow::{Error, anyhow};
use clap::{Args, Parser};
use std::{path::PathBuf, str::FromStr};

pub mod ast;
pub mod rust;
pub mod tsgen;
pub mod verify;

pub fn run_cli() -> i32 {
    let cli = Cli::parse();

    let r = match cli.command {
        Command::Verify(opts) => verify::verify(&opts),
        Command::Ast(opts) => ast::ast(&opts),
        Command::Rust(opts) => rust::rust(&opts),
        Command::Tsgen(opts) => tsgen::tsgen(&opts),
        Command::WriteStdlib(opts) => crate::adlstdlib::dump(&opts),
    };
    match r {
        Ok(_) => 0,
        Err(err) => {
            log::error!("{}", err);
            1
        }
    }
}

#[derive(Parser)]
#[command(name = "adlc")]
#[command(author = "Tim Docker")]
#[command(version = "0.1")]
#[command(about = "ADL code generation cli tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Parser)]
pub enum Command {
    /// verify ADL
    Verify(VerifyOpts),
    /// generate the json AST for some ADL modules
    Ast(AstOpts),
    /// generate rust code for the some ADL modules
    Rust(RustOpts),
    /// generate typescript code for the some ADL modules
    #[clap(name="typescript")]
    Tsgen(TsOpts),
    /// dump the embedded stdlib to the filesystem.
    WriteStdlib(DumpStdlibOpts),
}

#[derive(Debug, Args)]
pub struct DumpStdlibOpts {
    /// writes generated code to the specified directory
    #[arg(long, short='O', value_name="DIR")]
    pub outputdir: PathBuf,
}

#[derive(Debug, Args)]
pub struct VerifyOpts {
    #[clap(flatten)]
    pub search: AdlSearchOpts,

    pub modules: Vec<String>,
}

#[derive(Debug, Args)]
pub struct AstOpts {
    #[clap(flatten)]
    pub search: AdlSearchOpts,

    /// writes the AST to the specified file"
    #[arg(long, short='O', value_name="FILE")]
    pub outfile: Option<PathBuf>,

    pub modules: Vec<String>,
}

#[derive(Debug, Args)]
pub struct RustOpts {
    #[clap(flatten)]
    pub search: AdlSearchOpts,

    #[clap(flatten)]
    pub output: OutputOpts,

    /// Generate the runtime code
    #[arg(long)]
    pub include_runtime: bool,

    /// The module where the code is generated, relative to crate root
    #[arg(long, value_name="RSMODULE", default_value_t={"adl".to_string()})]
    pub module: String,

    /// The module where the runtime is located, relative to crate root
    #[arg(long, value_name="RSMODULE", default_value_t={"adlrt".to_string()})]
    pub runtime_module: String,

    #[arg(value_name="ADLMODULE")]
    pub modules: Vec<String>,
}


#[derive(Debug, Args)]
pub struct TsOpts {
    #[clap(flatten)]
    pub search: AdlSearchOpts,

    #[clap(flatten)]
    pub output: OutputOpts,

    /// Generate the runtime code
    #[arg(long)]
    pub include_rt: bool,

    /// Set the directory where runtime code is written (relative to output dir).
    #[arg(long, short='R', value_name="DIR")]
    pub runtime_dir: Option<String>,

    /// Also generate code for the transitive dependencies of the specified adl files (default: true)
    #[arg(long, default_value_t = true)]
    pub generate_transitive: bool,

    /// Generate the resolver map for all generated adl files (default: true)
    #[arg(long, default_value_t = true)]
    include_resolver: bool,

    /// Select the style of typescript to be generated
    // #[clap(arg_senum)]
    #[arg(long)]
    pub ts_style: Option<TsStyle>,//=tsc|deno

    #[arg(value_name="ADLMODULE")]
    pub modules: Vec<String>,

    /// If set capitalizes branch (field) name in the exported interfaces (used to generate backward code).
    ///
    /// Has a risk of creating name clashes between branches with only differ in case.
    /// Set to true to preserve backward compatiblity.
    #[arg(long)]
    pub capitalize_branch_names_in_types: bool,

    /// Capitalizes type names (default: true).
    #[arg(long, default_value_t = true)]
    pub capitalize_type_names: bool,

}


#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TsStyle {
    Tsc,
    Deno,
}

impl FromStr for TsStyle {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "tsc" => Ok(TsStyle::Tsc),
            "deno" => Ok(TsStyle::Deno),
            _ => Err(anyhow!("must be one of 'tsc' or 'deno'")),
        }
    }
}

#[derive(Debug, Args)]
pub struct AdlSearchOpts {
    /// adds the given directory to the ADL search path
    #[arg(long="searchdir", short='I', value_name="DIR")]
    pub path: Vec<PathBuf>,
}

#[derive(Debug, Args)]
pub struct OutputOpts {
    /// writes generated code to the specified directory
    #[arg(long, short='O', value_name="DIR")]
    pub outputdir: PathBuf,

    /// write a manifest file recording generated files
    #[arg(long, value_name="FILE")]
    pub manifest: Option<PathBuf>,
}

