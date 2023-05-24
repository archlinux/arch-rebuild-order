use clap::Parser;

use arch_rebuild_order::args::Args;

fn main() {
    let args = Args::parse();
    match arch_rebuild_order::run(
        args.pkgnames,
        args.dbpath,
        args.repos,
        args.dotfile,
        args.no_reverse_depends,
        args.with_check_depends,
    ) {
        Ok(output) => {
            println!("{output}");
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("Critical failure - arch-rebuild-order has stopped working");
            eprintln!("Reason: {}", e);
            std::process::exit(1);
        }
    }
}
