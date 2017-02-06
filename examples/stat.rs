extern crate cygwin_path;

use std::path::Path;

fn stat_path(path: &Path, cygpath: &cygwin_path::CygwinPath) {
    println!("{:?}", path);

    if !cygpath.running_under_cygwin() {
        println!("  Not running under cygwin.");
    } else {
        println!("  Cygwin root: {:?}", cygpath.root_path());

        let maybe_cygwin_symlink = cygwin_path::maybe_cygwin_symlink(&path);
        println!("  Maybe cygwin symlink: {}", maybe_cygwin_symlink);
    }
}

fn main() {
    let cygpath = cygwin_path::CygwinPath::new();

    let mut first_arg = true;
    for arg in std::env::args_os() {
        if first_arg { first_arg = false; continue; }
        let path = Path::new(&arg);
        stat_path(path, &cygpath);
    }
}
