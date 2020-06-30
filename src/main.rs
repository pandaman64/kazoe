use flate2::read::ZlibDecoder;
use gumdrop::Options;
use log::{error, info};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Options)]
struct Walk {
    #[options(free)]
    path: PathBuf,
    verbose: bool,
}

#[derive(Debug, Options)]
struct Libgit {
    #[options(free)]
    path: PathBuf,
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

                            let mut decoder = ZlibDecoder::new(file);
                            let mut content = Vec::new();
                            match decoder.read_to_end(&mut content) {
                                Ok(_) => {}
                                Err(e) => {
                                    error!("failed to read object file: {:?}", e);
                                    continue;
                                }
                            }
                            let header_end = match content.iter().position(|b| *b == 0) {
                                Some(n) => n,
                                None => {
                                    error!("malformed object: no NUL in the file");
                                    continue;
                                }
                            };
                            info!("{}", String::from_utf8_lossy(&content[0..header_end]));

                            if content.starts_with(b"blob") {
                                blob += 1;
                            }
                            if content.starts_with(b"tree") {
                                tree += 1;
                            }
                            if content.starts_with(b"commit") {
                                commit += 1;
                            }
                            if content.starts_with(b"tag") {
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

fn main() {
    let opt = Opt::parse_args_default_or_exit();
    let counts = match opt {
        Opt::Libgit(Libgit { path, verbose }) => todo!(),
        Opt::Walk(Walk { mut path, verbose }) => {
            if verbose {
                env_logger::init();
            }
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
