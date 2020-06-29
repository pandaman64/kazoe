use gumdrop::Options;
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Debug, Options)]
struct Opt {
    #[options(free)]
    path: PathBuf,
}

fn main() {
    let mut opts = Opt::parse_args_default_or_exit();
    println!("{:?}", opts);
    opts.path.push(".git");
    opts.path.push("objects");

    for path in WalkDir::new(opts.path) {
        println!("{:?}", path);
    }
}
