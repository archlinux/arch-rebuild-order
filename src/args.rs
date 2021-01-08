use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "arch-rebuild-order", about, author)]
pub struct Args {
    /// List of input packages
    #[structopt(min_values = 1, required = true)]
    pub pkgnames: Vec<String>,

    /// Repositories
    #[structopt(
        default_value = "core,extra,community,multilib",
        long,
        use_delimiter = true
    )]
    pub repos: Vec<String>,

    /// The path to the pacman database, default ( /var/lib/pacman )
    #[structopt(long)]
    pub dbpath: Option<String>,

    /// Write a dotfile into the given file
    #[structopt(short, long)]
    pub dotfile: Option<String>,
}
