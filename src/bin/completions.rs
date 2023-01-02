use clap::{CommandFactory, ValueEnum};
use clap_complete::Shell;
use std::env;

use arch_rebuild_order::args::Args;

fn main() -> anyhow::Result<()> {
    //  https://doc.rust-lang.org/cargo/reference/environment-variables.html
    let out_dir = env::var_os("OUT_DIR").expect("out dir not set");
    for variant in Shell::value_variants() {
        clap_complete::generate_to(
            *variant,
            &mut Args::command(),
            "arch-rebuild-order",
            &out_dir,
        )?;
    }
    println!("completion scripts generated in {:?}", out_dir);
    Ok(())
}
