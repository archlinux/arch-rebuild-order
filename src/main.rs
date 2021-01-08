use structopt::StructOpt;

use arch_rebuild_order::args::Args;

fn main() {
    let args = Args::from_args();
    match arch_rebuild_order::run(args.pkgnames, args.dbpath, args.repos, args.dotfile) {
        Ok(output) => {
            println!("{}", output);
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("Critical failure - arch-rebuild-order has stopped working");
            eprintln!("Reason: {}", e);
            std::process::exit(1);
        }
    }
}
