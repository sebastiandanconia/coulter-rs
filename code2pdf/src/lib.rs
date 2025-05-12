use std::error::Error;
use std::io::{Write};
use std::process::{Command};

pub mod config;
pub use config::*;

const ENSCRIPT: &str = "enscript";
const PS2PDF: & str = "ps2pdf";


// Convert a string containing user code into a PDF
pub fn pretty_print(context: EnscriptContext) -> Result<String, Box<dyn Error>> {

    let mut txtfile = tempfile::NamedTempFile::new()?;

    txtfile.write_all(context.code().as_bytes())?;

    let txt_path = txtfile.path().to_str().unwrap(); // FIXME: ok_or(InvalidData)?;

    let phantom_ps = tempfile::Builder::new()
        .prefix("pretty-print-")
        .suffix(".ps")
        .make(|_| Ok(()))?;
    let ps_path = phantom_ps.path().to_str().unwrap().to_owned();

    let phantom_pdf = tempfile::Builder::new()
        .prefix("pretty-print-")
        .suffix(".pdf")
        .rand_bytes(6)
        .make(|_| Ok(()))?; // Skip the actual file creation step.

    let pdf_path = phantom_pdf.path().to_str().unwrap().to_owned();

    let mut options: Vec<String> = Vec::<String>::new();

        let orientation = match context.orientation() {
            Orientation::Portrait => "--portrait",
            Orientation::Landscape => "--landscape",
        };

        let header;
        if context.title().is_some() {
            header = "--header=%H| %W |$%";
        } else {
            header = "--header=%W |$%";
        }

        let syntax = match context.syntax() {
            Some(lang) => format!("--highlight={}", lang.as_str()),
            None => String::default(),
        };

        options.extend([
            "--no-job-header".into(),
            "--line-numbers".into(),

            format!("--title={}", context.title().unwrap_or("Untitled")),
            header.into(),
            format!("--media={}", context.media().as_str()),
            orientation.into(),
            format!("--tabsize={0}", context.tabsize()),
            format!("--columns={0}", context.columns()),
            format!("--font={0}", context.font()),
            format!("--color={}", if context.color() { 1 } else { 0 }),
            syntax,

            "-p".into(), ps_path.to_owned(), txt_path.to_owned()
            ]);

    // println!("Running `{} {}'", ENSCRIPT, options.join(" "));
    run_command(ENSCRIPT, &options)?;

    run_command(PS2PDF, &[
        ps_path.to_owned(),
        pdf_path.to_owned()])?;

    phantom_pdf.keep()?;
    println!("Check output PDF: {pdf_path}");

    Ok(pdf_path.to_string())
}

// Run a command
fn run_command(cmd: &str, args: &[String]) -> Result<(), String> {
    let mut child = Command::new(cmd);
    child.args(args);

    let status = child.status();
    println!("Running `{} {}'", cmd, args.join(" "));
    match status {
        Err(s) => Err(format!("Error running `{} {}': {}", cmd, args.join(" "), s)),
        Ok(s) => match s.success() {
            true => Ok(()),
            false => Err(format!("`{} {}' exited with non-zero status", cmd, args.join(" ")))
        }
    }
}