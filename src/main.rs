extern crate cargo;
extern crate rustc_serialize;
extern crate docopt;

#[macro_use]
mod common;
mod resolve;
mod scan;

use cargo::util::Config;
use docopt::Docopt;
use rustc_serialize::json;

const USAGE: &'static str = "
Usage: srclib-rust scan [--repo=<repo>] [--subdir=<subdir>]
";

fn main() {
  Docopt::new(USAGE).and_then(|dopt| dopt.parse())
                    .unwrap_or_else(|e| e.exit());
  
  let config = Config::default().unwrap();

  // Assume scan
  let source_units = scan::execute(&config).unwrap();
  let encoded = json::encode(&source_units).unwrap();
  println!("{}", encoded);
}