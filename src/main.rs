use flate2::read::ZlibDecoder;
use gumdrop::Options;
use log::{error, info};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Debug, Options)]
struct Opt {
    #[options(free)]
    path: PathBuf,
    verbose: bool,
}

fn main() {
    let mut opts = Opt::parse_args_default_or_exit();
    if opts.verbose {
        env_logger::init();
    }
    info!("{:?}", opts);
    opts.path.push(".git");
    opts.path.push("objects");

    for path in WalkDir::new(opts.path) {
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
                            println!("{}", String::from_utf8_lossy(&content[0..header_end]));
                        }
                    }
                    Err(e) => error!("metadata error {:?}", e),
                }
            }
            Err(e) => error!("walkdir error {:?}", e),
        }
    }
}
