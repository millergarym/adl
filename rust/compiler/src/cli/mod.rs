use gumdrop::Options;
use std::path::PathBuf;

pub mod ast;
pub mod verify;

pub fn run_cli() -> i32 {
    let opts = CliOptions::parse_args_default_or_exit();

    let r = match opts.command {
        None => {
            println!("{}", CliOptions::self_usage(&opts));
            return 0;
        }
        Some(Command::Verify(opts)) => {
            if opts.modules.len() == 0 {
                println!("{}", VerifyOpts::self_usage(&opts));
                return 1;
            } else {
                match verify::verify(&opts) {
                    Ok(_) => 0,
                    Err(_err) => 2,
                }
            }
        }
        Some(Command::Ast(opts)) => {
            if opts.modules.len() == 0 {
                println!("{}", AstOpts::self_usage(&opts));
                return 1;
            } else {
                match ast::ast(&opts) {
                    Ok(_) => 0,
                    Err(_err) => 2,
                }
            }
        }
    };
    return r;
    // match r {
    //     Ok(_) => 0,
    //     Err(err) => {
    //         log::error!("{}", err);
    //         1
    //     }
    // }
}

// Define options for the program.
#[derive(Debug, Options)]
pub struct CliOptions {
    #[options(help = "print help message")]
    pub help: bool,
    #[options(command)]
    pub command: Option<Command>,
}

#[derive(Debug, Options)]
pub enum Command {
    #[options(help = "verify ADL")]
    Verify(VerifyOpts),
    #[options(help = "generate the json AST for some ADL")]
    Ast(AstOpts),
}

#[derive(Debug, Options)]
pub struct VerifyOpts {
    #[options(help = "print help message")]
    pub help: bool,
    #[options(help = "adds the given directory to the ADL search path", meta = "I")]
    pub searchdir: Vec<PathBuf>,
    #[options(free)]
    pub modules: Vec<String>,
}

#[derive(Debug, Options)]
pub struct AstOpts {
    #[options(help = "print help message")]
    pub help: bool,
    #[options(help = "adds the given directory to the ADL search path", meta = "I")]
    pub searchdir: Vec<PathBuf>,

    #[options(help = "writes the AST to the specified file", meta = "O")]
    pub outfile: Option<PathBuf>,

    #[options(free)]
    pub modules: Vec<String>,
}
