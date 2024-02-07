use std::error::Error;
use std::io::{Write};
use std::process::{Command};

const ENSCRIPT: &str = "enscript";
const PS2PDF: & str = "ps2pdf";


#[derive(Debug,PartialEq)]
enum Orientation {
    Portrait,
    Landscape,
}

#[derive(Debug,PartialEq)]
pub struct Config {
    // filename: Option<String>,
    title: String,
    media: String,
    syntax: Option<String>,
    font: String,
    color: bool,
    tabsize: u32,
    columns: u32,
    orientation: Orientation,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            title: "Untitled".into(),
            media: "Letter".into(),
            syntax: None,
            font: "Courier7".into(),
            color: true,
            tabsize: 4,
            columns: 2,
            orientation: Orientation::Landscape
        }
    }
}

// Convert a string containing user code into a PDF
pub fn pretty_print(config: Config, code: &str) -> Result<String, Box<dyn Error>> {

    let mut txtfile = tempfile::NamedTempFile::new()?;

    txtfile.write_all(code.as_bytes())?;

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
        .make(|_| Ok(()))?; // Skip the actual file creation step for now.

    let pdf_path = phantom_pdf.path().to_str().unwrap().to_owned();

    let mut options: Vec<String> = vec!["--no-job-header".into(), "--line-numbers".into(), "--landscape".into(),
        format!("--media={}", config.media), format!("--title={}", config.title), "--header=%W |$%".into()];
        options.extend([format!("--tabsize={0}", config.tabsize), format!("--columns={0}", config.columns)]);
        options.extend(["--font=Courier7".into(), format!("--color={}", config.color)]);
        options.extend(["-p".into(), ps_path.to_owned(), txt_path.to_owned()]);
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