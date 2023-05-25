use rstest::fixture;
use std::convert::TryFrom;
use std::fmt;
use std::fs;
use std::fs::File;
use std::io::Write;
use tar::{Builder, Header};
use tempfile::{tempdir, TempDir};

// pacman database version (lib/libalpm/be_local.c)
const ALPM_DB_VERSION: &str = "9";

#[derive(Hash, Clone, Eq, PartialEq, Debug)]
pub struct Package {
    pub name: String,
    pub base: String,
    pub version: String,
    pub depends: Vec<String>,
    pub makedepends: Vec<String>,
    pub checkdepends: Vec<String>,
    pub provides: Vec<String>,
}

impl Package {
    fn new(
        name: &str,
        base: &str,
        version: &str,
        depends: Vec<String>,
        makedepends: Vec<String>,
        provides: Vec<String>,
        checkdepends: Vec<String>,
    ) -> Package {
        Package {
            name: name.to_string(),
            base: base.to_string(),
            version: version.to_string(),
            depends,
            makedepends,
            provides,
            checkdepends,
        }
    }

    fn desc(&self) -> String {
        let mut desc = String::from("");

        let name = format!("%NAME%\n{}\n", self.name);
        desc.push_str(&name);

        let base = format!("%BASE%\n{}\n", self.base);
        desc.push_str(&base);

        if !self.depends.is_empty() {
            desc.push_str("%DEPENDS%\n");
            for dep in self.depends.iter() {
                desc.push_str(dep);
                desc.push_str("\n");
            }
            desc.push_str("\n");
        }

        if !self.checkdepends.is_empty() {
            desc.push_str("%CHECKDEPENDS%\n");
            for dep in self.checkdepends.iter() {
                desc.push_str(dep);
                desc.push_str("\n");
            }
            desc.push_str("\n");
        }

        if !self.makedepends.is_empty() {
            desc.push_str("%MAKEDEPENDS%\n");
            for dep in self.makedepends.iter() {
                desc.push_str(dep);
                desc.push_str("\n");
            }
            desc.push_str("\n");
        }

        if !self.provides.is_empty() {
            desc.push_str("%PROVIDES%\n");
            for dep in self.provides.iter() {
                desc.push_str(dep);
                desc.push_str("\n");
            }
            desc.push_str("\n");
        }

        desc
    }

    fn path(&self) -> String {
        format!("{}-{}/desc", self.name, self.version)
    }

    fn tarheader(&self) -> Header {
        let mut header = Header::new_gnu();
        let desc = self.desc();
        let datalen = u64::try_from(desc.len()).unwrap();
        header.set_path(self.path()).unwrap();
        header.set_size(datalen);
        header.set_mode(0o644);
        header.set_cksum();

        header
    }
}

impl fmt::Display for Package {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl PartialEq<&str> for Package {
    fn eq(&self, other: &&str) -> bool {
        &self.name.as_str() == other
    }
}

fn init_repodb(reponame: String, packages: Vec<Package>) -> (TempDir, String) {
    let tempdir = tempdir().unwrap();
    let dbpath = tempdir.path().display().to_string();

    // local dir
    let localdir = tempdir.path().join("local");
    fs::create_dir(&localdir).unwrap();

    let mut file = File::create(localdir.join("ALPM_DB_VERSION")).unwrap();
    file.write_all(ALPM_DB_VERSION.as_bytes()).unwrap();

    // sync dir
    let syncdir = tempdir.path().join("sync");
    fs::create_dir(&syncdir).unwrap();

    let dbloc = syncdir.join(format!("{}.db", reponame));
    create_db(dbloc.display().to_string(), packages);

    (tempdir, dbpath)
}

fn create_db(dbloc: String, pkgs: Vec<Package>) {
    let mut archive = Builder::new(Vec::new());

    for pkg in pkgs {
        let header = pkg.tarheader();
        let desc = pkg.desc();
        let data = desc.as_bytes();
        archive.append(&header, data).unwrap();
    }

    archive.finish().unwrap();
    let data = archive.into_inner().unwrap();

    let mut afile = File::create(dbloc).unwrap();
    afile.write_all(&data).unwrap();
}

#[fixture]
pub fn invalid_dbpath() -> (Vec<String>, Option<String>) {
    let pkgnames = vec![String::from("testpkg1")];
    let dbpath = Some(String::from("/non-existant-path"));

    (pkgnames, dbpath)
}

#[fixture]
pub fn no_reverse_deps() -> (Vec<Package>, Option<String>, Vec<String>, TempDir) {
    let testpkg = Package::new(
        "testpkg1",
        "testpkg1",
        "1.0-1",
        vec![],
        vec![],
        vec![],
        vec![],
    );
    let packages = vec![testpkg];

    let reponame = "test";
    let (rootdir, dbpath) = init_repodb(reponame.to_string(), packages.clone());

    (packages, Some(dbpath), vec![reponame.to_string()], rootdir)
}

#[fixture]
pub fn reverse_deps() -> (Vec<String>, Option<String>, Vec<String>, TempDir) {
    let testpkg = Package::new(
        "testpkg1",
        "testpkg1",
        "1-1",
        vec![],
        vec![],
        vec![],
        vec![],
    );
    let testpkg2 = Package::new(
        "testpkg2",
        "testpkg2",
        "1-1",
        vec![testpkg.name.clone()],
        vec![],
        vec![],
        vec![],
    );
    let pkgnames = vec![testpkg.name.clone(), testpkg2.name.clone()];
    let packages = vec![testpkg, testpkg2];

    let reponame = "test";
    let (tempdir, dbpath) = init_repodb(reponame.to_string(), packages);

    (pkgnames, Some(dbpath), vec![reponame.to_string()], tempdir)
}

#[fixture]
pub fn multiple_deps() -> (Vec<Package>, Option<String>, Vec<String>, TempDir) {
    let testpkg = Package::new(
        "testpkg1",
        "testpkg1",
        "1.0-1",
        vec![],
        vec![],
        vec![],
        vec![],
    );
    let testpkg2 = Package::new(
        "testpkg2",
        "testpkg2",
        "1.0-1",
        vec![testpkg.name.clone()],
        vec![],
        vec![],
        vec![],
    );
    let testpkg3 = Package::new(
        "testpkg3",
        "testpkg3",
        "1-1",
        vec![testpkg.name.clone(), testpkg2.name.clone()],
        vec![],
        vec![],
        vec![],
    );
    let packages = vec![testpkg3, testpkg2, testpkg];

    let reponame = "test";
    let (tempdir, dbpath) = init_repodb(reponame.to_string(), packages.clone());

    (packages, Some(dbpath), vec![reponame.to_string()], tempdir)
}

#[fixture]
pub fn reverse_make_deps() -> (Vec<Package>, Option<String>, Vec<String>, TempDir) {
    let testpkg = Package::new(
        "testpkg1",
        "testpkg1",
        "1-1",
        vec![],
        vec![],
        vec![],
        vec![],
    );
    let testpkg2 = Package::new(
        "testpkg2",
        "testpkg2",
        "1-1",
        vec![],
        vec![testpkg.name.clone()],
        vec![],
        vec![],
    );
    let packages = vec![testpkg, testpkg2];

    let reponame = "test";
    let (tempdir, dbpath) = init_repodb(reponame.to_string(), packages.clone());

    (packages, Some(dbpath), vec![reponame.to_string()], tempdir)
}

#[fixture]
pub fn reverse_check_deps() -> (Vec<Package>, Option<String>, Vec<String>, TempDir) {
    let testpkg = Package::new(
        "testpkg1",
        "testpkg1",
        "1-1",
        vec![],
        vec![],
        vec![],
        vec![],
    );
    let testpkg2 = Package::new(
        "testpkg2",
        "testpkg2",
        "1-1",
        vec![],
        vec![],
        vec![],
        vec![testpkg.name.clone()],
    );
    let packages = vec![testpkg, testpkg2];

    let reponame = "test";
    let (tempdir, dbpath) = init_repodb(reponame.to_string(), packages.clone());

    (packages, Some(dbpath), vec![reponame.to_string()], tempdir)
}

#[fixture]
pub fn provides_make_depends() -> (Vec<Package>, Option<String>, Vec<String>, TempDir) {
    let testpkg = Package::new(
        "testpkg1",
        "testpkg1",
        "1-1",
        vec![],
        vec![],
        vec!["pkg1".to_string()],
        vec![],
    );
    let testpkg2 = Package::new(
        "testpkg2",
        "testpkg2",
        "1-1",
        vec![],
        vec![testpkg.provides[0].clone()],
        vec![],
        vec![],
    );
    let packages = vec![testpkg, testpkg2];

    let reponame = "test";
    let (tempdir, dbpath) = init_repodb(reponame.to_string(), packages.clone());

    (packages, Some(dbpath), vec![reponame.to_string()], tempdir)
}

#[fixture]
pub fn dependency_depth() -> (Vec<Package>, Option<String>, Vec<String>, TempDir) {
    let testpkg = Package::new(
        "testpkg1",
        "testpkg1",
        "1-1",
        vec![],
        vec![],
        vec![],
        vec![],
    );
    let testpkg2 = Package::new(
        "testpkg2",
        "testpkg2",
        "1-1",
        vec![],
        vec![testpkg.name.clone()],
        vec![],
        vec![],
    );
    let testpkg3 = Package::new(
        "testpkg3",
        "testpkg3",
        "1-1",
        vec![],
        vec![testpkg2.name.clone()],
        vec![],
        vec![],
    );
    let packages = vec![testpkg, testpkg2, testpkg3];

    let reponame = "test";
    let (tempdir, dbpath) = init_repodb(reponame.to_string(), packages.clone());

    (packages, Some(dbpath), vec![reponame.to_string()], tempdir)
}

#[fixture]
pub fn dependency_cycle() -> (Vec<Package>, Option<String>, Vec<String>, TempDir) {
    let testpkg = Package::new(
        "testpkg1",
        "testpkg1",
        "1-1",
        vec![String::from("testpkg2")],
        vec![],
        vec![],
        vec![],
    );
    let testpkg2 = Package::new(
        "testpkg2",
        "testpkg2",
        "1-1",
        vec![testpkg.name.clone()],
        vec![],
        vec![],
        vec![],
    );
    let packages = vec![testpkg, testpkg2];

    let reponame = "test";
    let (tempdir, dbpath) = init_repodb(reponame.to_string(), packages.clone());

    (packages, Some(dbpath), vec![reponame.to_string()], tempdir)
}

#[fixture]
pub fn multiple_pkgnames() -> (Vec<Package>, Option<String>, Vec<String>, TempDir) {
    let testpkg = Package::new(
        "testpkg1",
        "testpkg1",
        "1-1",
        vec![],
        vec![],
        vec![],
        vec![],
    );
    let testpkg2 = Package::new(
        "testpkg2",
        "testpkg2",
        "1-1",
        vec![testpkg.name.clone()],
        vec![],
        vec![],
        vec![],
    );
    let testpkg3 = Package::new(
        "testpkg3",
        "testpkg3",
        "1-1",
        vec![testpkg.name.clone()],
        vec![],
        vec![],
        vec![],
    );
    let testpkg4 = Package::new(
        "testpkg4",
        "testpkg4",
        "1-1",
        vec![testpkg2.name.clone()],
        vec![],
        vec![],
        vec![],
    );

    let packages = vec![testpkg, testpkg2, testpkg3, testpkg4];

    let reponame = "test";
    let (tempdir, dbpath) = init_repodb(reponame.to_string(), packages.clone());

    (packages, Some(dbpath), vec![reponame.to_string()], tempdir)
}
