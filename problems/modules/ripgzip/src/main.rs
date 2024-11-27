#![forbid(unsafe_code)]

use std::fs::File;
use std::io::{stdin, stdout, BufReader};

use log::*;
use structopt::StructOpt;

use ripgzip::decompress;

#[derive(StructOpt, Debug)]
#[structopt()]
struct Opts {
    /// Decompress data
    #[structopt(short = "d", long = "decompress")]
    decompress: bool,
    /// Verbose mode (-v, -vv, -vvv, etc)
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    verbose: usize,
}
//06-war-and-peace.txt.gz
fn main() {
   /* let c = std::env::current_dir().unwrap();
    let ck = c.join("problems/modules/ripgzip");
    let file = File::open(ck.join("data/ok/09-concat.gz")).unwrap();
    let output_file = File::create(ck.join("concat.pdf")).unwrap();
    let res = decompress(BufReader::new(file), output_file);
    if res.is_err() {
        panic!("{}", res.unwrap_err())
    }*/
    let opts = Opts::from_args();
    stderrlog::new()
        .verbosity(1 + opts.verbose)
        .timestamp(stderrlog::Timestamp::Off)
        .init()
        .expect("failed to initialize logging");

    if opts.decompress {
        if let Err(err) = decompress(stdin().lock(), stdout().lock()) {
            error!("{:#}", err);
            std::process::exit(1);
        }
    }
}
