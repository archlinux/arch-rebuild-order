use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "arch-rebuild-order", about, author)]
struct Args {
    /// List of input packages
    #[structopt(min_values = 1, required = true)]
    pkgnames: Vec<String>,

    /// Repositories
    #[structopt(
        default_value = "core,extra,community,multilib",
        long,
        use_delimiter = true
    )]
    repos: Vec<String>,

    /// The path to the pacman database, default ( /var/lib/pacman )
    #[structopt(long)]
    dbpath: Option<String>,

    /// Write a dotfile into the given file
    #[structopt(short, long)]
    dotfile: Option<String>,
}

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
