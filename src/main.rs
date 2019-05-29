extern crate failure;
extern crate failure_tools;
extern crate dua;
#[macro_use]
extern crate structopt;

use failure::Error;
use failure_tools::ok_or_exit;
use structopt::StructOpt;

mod options {
    use std::path::PathBuf;

    #[derive(Debug, StructOpt)]
    #[structopt(name = "example", about = "An example of StructOpt usage.")]
    pub struct Args {
        /// Activate debug mode
        #[structopt(short = "d", long = "debug")]
        debug: bool,
        /// Set speed
        #[structopt(short = "s", long = "speed", default_value = "42")]
        speed: f64,
        /// Input file
        #[structopt(parse(from_os_str))]
        input: PathBuf,
        /// Output file, stdout if not present
        #[structopt(parse(from_os_str))]
        output: Option<PathBuf>,
    }
}

fn run() -> Result<(), Error> {
    let opt = options::Args::from_args();
    println!("{:?}", opt);
    dua::fun()
}

fn main() {
    ok_or_exit(run())
}
