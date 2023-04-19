use std::path::PathBuf;

use anyhow::{anyhow, Error};
use clap::{Args, Parser};
use std::str::FromStr;

use crate::{
    adlgen::adlc::packaging::{
        GenOutput, ModuleSrc, NpmPackageRef, ReferenceableScopeOption, TsGenRuntime, TsRuntimeOpt,
        TypescriptGenOptions,
    },
    processing::loader::loader_from_search_paths,
};
// use std::path{Path, PathBuf};

pub mod ast;
pub mod rust;
pub mod tsgen;
pub mod verify;
pub mod workspace;

pub fn run_cli() -> i32 {
    let cli = Cli::parse();

    let r = match cli.command {
        Command::Gen(opts) => workspace::workspace(&opts),
        Command::Verify(opts) => verify::verify(&opts),
        Command::Ast(opts) => ast::ast(&opts),
        Command::Rust(opts) => rust::rust(&opts),
        Command::Tsgen(opts) => {
            if let Some(d) = opts.runtime_dir {
                if d != "./runtime" {
                    eprintln!("only value for runtime_dir which is supported in './runtime'");
                    return 1;
                }
            }
            let loader = loader_from_search_paths(&opts.search.path);
            let ts_opts = TypescriptGenOptions {
                npm_pkg_name: opts.npm_pkg_name,
                npm_version: TypescriptGenOptions::def_npm_version(),
                extra_dependencies: TypescriptGenOptions::def_extra_dependencies(),
                extra_dev_dependencies: TypescriptGenOptions::def_extra_dev_dependencies(),
                annotate: vec![],
                outputs: Some(crate::adlgen::adlc::packaging::OutputOpts::Gen(GenOutput {
                    referenceable: ReferenceableScopeOption::Local,
                    output_dir: opts.output.outputdir.to_str().unwrap().to_string(),
                    manifest: opts
                        .output
                        .manifest
                        .map(|m| m.to_str().unwrap().to_string()),
                })),
                runtime_opts: if opts.include_rt {
                    TsRuntimeOpt::Generate(TsGenRuntime{
                        // runtime_dir: match opts.runtime_dir {
                        //     Some(d) => d,
                        //     None => "runtime".to_string(),
                        // },
                    })
                } else {
                    match opts.runtime_pkg {
                        Some(d) => TsRuntimeOpt::PackageRef(NpmPackageRef {
                            name: d,
                            version: "^1.0.0".to_string(),
                        }),
                        None => TypescriptGenOptions::def_runtime_opts(),
                    }
                },
                generate_transitive: opts.generate_transitive,
                include_resolver: opts.include_resolver,
                ts_style: match opts.ts_style {
                    Some(style) => match style {
                        TsStyle::Tsc => crate::adlgen::adlc::packaging::TsStyle::Tsc,
                        TsStyle::Deno => crate::adlgen::adlc::packaging::TsStyle::Deno,
                    },
                    None => crate::adlgen::adlc::packaging::TsStyle::Tsc,
                },
                modules: ModuleSrc::Modules(opts.modules),
                capitalize_branch_names_in_types: opts.capitalize_branch_names_in_types,
                capitalize_type_names: opts.capitalize_type_names,
            };
            tsgen::tsgen(loader, &ts_opts, None)
        }
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
    /// generate source based on Workspace & Packages files (adl.work.json & adl.pkg.json)
    Gen(GenOpts),
    /// verify ADL
    Verify(VerifyOpts),
    /// generate the json AST for some ADL modules
    Ast(AstOpts),
    /// generate rust code for the some ADL modules
    Rust(RustOpts),
    /// generate typescript code for the some ADL modules
    #[clap(name = "typescript")]
    Tsgen(TsOpts),
    /// dump the embedded stdlib to the filesystem.
    WriteStdlib(DumpStdlibOpts),
}

#[derive(Debug, Args)]
pub struct GenOpts {
    /// The module where the code is generated, relative to crate root
    #[arg(default_value_t={".".to_string()})]
    pub dir: String,

    /// The workspace file to use relative to the dir
    #[arg(long, short='f', default_value_t={"adl.work.json".to_string()})]
    pub workspace_filename: String,

    /// The package filenames to look for in the pkg dir specified in the use fields
    #[arg(long, short='p', default_value_t={"adl.pkg.json".to_string()})]
    pub package_filenames: String,
}

#[derive(Debug, Args)]
pub struct DumpStdlibOpts {
    /// writes generated code to the specified directory
    #[arg(long, short = 'O', value_name = "DIR")]
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
    #[arg(long, short = 'O', value_name = "FILE")]
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

    #[arg(value_name = "ADLMODULE")]
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
    #[arg(long, short = 'R', value_name = "DIR")]
    pub runtime_dir: Option<String>,

    #[arg(long)]
    pub runtime_pkg: Option<String>,

    /// Also generate code for the transitive dependencies of the specified adl files (default: true)
    #[arg(long, default_value_t = true)]
    pub generate_transitive: bool,

    /// The name to use in the package.json file
    #[arg(long, default_value_t = String::from("my_data"))]
    pub npm_pkg_name: String,

    /// Generate the resolver map for all generated adl files (default: true)
    #[arg(long, default_value_t = true)]
    include_resolver: bool,

    /// Select the style of typescript to be generated
    // #[clap(arg_senum)]
    #[arg(long)]
    pub ts_style: Option<TsStyle>, //=tsc|deno

    #[arg(value_name = "ADLMODULE")]
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
    #[arg(long = "searchdir", short = 'I', value_name = "DIR")]
    pub path: Vec<PathBuf>,
}

#[derive(Debug, Args)]
pub struct OutputOpts {
    /// writes generated code to the specified directory
    #[arg(long, short = 'O', value_name = "DIR")]
    pub outputdir: PathBuf,

    /// write a manifest file recording generated files
    #[arg(long, value_name = "FILE")]
    pub manifest: Option<PathBuf>,
}
