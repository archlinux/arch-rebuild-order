# NAME

arch-rebuild-order - Rebuild order generation script

# SYNOPSIS

**arch-rebuild-order** [OPTION]... [PACKAGES]...

# DESCRIPTION

Generate a rebuild order for given packages using pacman's local syncdb's.

**--d=FILE, --dotfile=FILE** Generate a .dot graph file with the rebuild order of the gives packages

**--dbpath=PATH** the path to pacman's database path

**--repos=REPOS** the repositories to retrieve the package information from

**--no-reverse-depends** only use pkgnames provided as input to calculate the build order, does not expand reverse (make)dependencies

**-V, --version** prints version information

**-h, --help** prints help information

# EXAMPLES

Generating an image of the rebuild order of provided package(s):

$ **arch-rebuild-order** -d linux-rebuild-order.dot linux


$ dot -Tpng linux-rebuild-order.dot > linux-rebuild-order.png

# BUGS

[Bug tracker](https://gitlab.archlinux.org/archlinux/arch-rebuild-order/-/issues)
