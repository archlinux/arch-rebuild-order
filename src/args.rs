#[derive(Debug, clap::Parser)]
#[clap(name = "arch-rebuild-order", about, author)]
pub struct Args {
    /// List of input packages
    #[arg(required = true)]
    pub pkgnames: Vec<String>,

    /// Repositories
    #[arg(
        default_value = "core,extra,community,multilib",
        long,
        use_value_delimiter = true
    )]
    pub repos: Vec<String>,

    /// The path to the pacman database, default ( /var/lib/pacman )
    #[arg(long)]
    pub dbpath: Option<String>,

    /// Write a dotfile into the given file
    #[arg(short, long)]
    pub dotfile: Option<String>,

    /// Only use the pkgnames provided as input
    #[arg(long)]
    pub no_reverse_depends: bool,
}
