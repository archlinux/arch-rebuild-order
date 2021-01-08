use std::{env, str::FromStr};
use structopt::clap::Shell;
use structopt::StructOpt;

use arch_rebuild_order::args::Args;

fn main() {
    //  https://doc.rust-lang.org/cargo/reference/environment-variables.html
    let out_dir = env::var_os("OUT_DIR").expect("out dir not set");
    let mut app = Args::clap();
    for variant in &Shell::variants() {
        let variant = Shell::from_str(variant).unwrap();
        app.gen_completions("arch-rebuild-order", variant, &out_dir);
    }
    println!("completion scripts generated in {:?}", out_dir);
}
