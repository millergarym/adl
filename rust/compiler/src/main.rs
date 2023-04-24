use compiler::cli::run_cli;
use log::LevelFilter;
use std::io::Write;

fn main() {
    // // useful for recursive stack overflows
    // unsafe { backtrace_on_stack_overflow::enable() };

    env_logger::builder()
        // .format_module_path(true)
        .format(|buf, record| {
            if let (Some(f), Some(l)) = (record.file(), record.line()) {
                writeln!(buf, "[{} {}:{}] {}", record.level(), f, l, record.args() )
            } else if let Some(m) = record.module_path() {
                writeln!(buf, "[{} {}] {}", record.level(), m , record.args() )
            } else {
                writeln!(buf, "[{}] {}", record.level(), record.args() )
            }
        })
        .filter(None, LevelFilter::Info)
        .init();

    // env_logger::init();
    let exit_code = run_cli();
    std::process::exit(exit_code);
}
