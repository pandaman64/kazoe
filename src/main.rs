use flate2::read::ZlibDecoder;
use git2::{ObjectType, Repository};
use gumdrop::Options;
use log::{error, info};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Options)]
struct Walk {
    #[options(free)]
    path: Option<PathBuf>,
    verbose: bool,
}

#[derive(Debug, Options)]
struct Libgit {
    #[options(free)]
    path: Option<PathBuf>,
    verbose: bool,
}

#[derive(Debug, Options)]
enum Opt {
    Walk(Walk),
    Libgit(Libgit),
}

#[derive(Debug)]
struct Counts {
    blob: usize,
    tree: usize,
    commit: usize,
    tag: usize,
}

fn walk_objects(path: &Path) -> Counts {
    let mut blob = 0;
    let mut tree = 0;
    let mut commit = 0;
    let mut tag = 0;

    for path in WalkDir::new(&path) {
        match path {
            Ok(entry) => {
                info!("{:?}", entry.path());
                match entry.metadata() {
                    Ok(metadata) => {
                        if metadata.is_file() {
                            let file = match File::open(entry.path()) {
                                Ok(file) => file,
                                Err(e) => {
                                    error!("failed to open file {:?}, {:?}", entry.path(), e);
                                    continue;
                                }
                            };

                            let mut decoder =
                                BufReader::with_capacity(1024, ZlibDecoder::new(file));
                            let mut header = Vec::new();
                            match decoder.read_until(0, &mut header) {
                                Ok(_) => {}
                                Err(e) => {
                                    error!("failed to read object file: {:?}", e);
                                    continue;
                                }
                            }
                            info!("{}", String::from_utf8_lossy(&header));

                            if header.starts_with(b"blob") {
                                blob += 1;
                            }
                            if header.starts_with(b"tree") {
                                tree += 1;
                            }
                            if header.starts_with(b"commit") {
                                commit += 1;
                            }
                            if header.starts_with(b"tag") {
                                tag += 1;
                            }
                        }
                    }
                    Err(e) => error!("metadata error {:?}", e),
                }
            }
            Err(e) => error!("walkdir error {:?}", e),
        }
    }

    Counts {
        blob,
        tree,
        commit,
        tag,
    }
}

// enumerate objects using libgit
fn foreach_objects(path: &Path) -> Counts {
    let mut blob = 0;
    let mut tree = 0;
    let mut commit = 0;
    let mut tag = 0;

    let repo = Repository::discover(path).unwrap();
    let odb = repo.odb().unwrap();
    odb.foreach(|oid| {
        match match odb.read_header(*oid) {
            Ok(oid) => oid,
            Err(e) => {
                error!("failed to read header {:?}", e);
                return true;
            }
        }
        .1
        {
            ObjectType::Blob => blob += 1,
            ObjectType::Tree => tree += 1,
            ObjectType::Commit => commit += 1,
            ObjectType::Tag => tag += 1,
            ObjectType::Any => error!("encountered any object"),
        }
        true
    })
    .unwrap();

    Counts {
        blob,
        tree,
        commit,
        tag,
    }
}

fn main() {
    let opt = Opt::parse_args_default_or_exit();
    let counts = match opt {
        Opt::Libgit(Libgit { path, verbose }) => {
            if verbose {
                env_logger::init();
            }

            let path = path.unwrap_or_else(|| ".".into());
            foreach_objects(&path)
        }
        Opt::Walk(Walk { path, verbose }) => {
            if verbose {
                env_logger::init();
            }
            let mut path = path.unwrap_or_else(|| ".".into());
            path.push(".git");
            path.push("objects");

            walk_objects(&path)
        }
    };
    println!("blob   {}", counts.blob);
    println!("tree   {}", counts.tree);
    println!("commit {}", counts.commit);
    println!("tag    {}", counts.tag);
}
