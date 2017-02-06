extern crate cygwin_fs;

use cygwin_fs::{CygRoot, maybe_cygwin_symlink};

use std::path::Path;

fn stat_path(path: &Path, cygroot: &CygRoot) {
    println!("{:?}", path);

    if !cygroot.running_under_cygwin() {
        println!("  Not running under cygwin.");
    } else {
        println!("  Cygwin root: {:?}", cygroot.root_path());

        println!("  Maybe cygwin symlink: {}", maybe_cygwin_symlink(&path));
    }
}

fn main() {
    let cygroot = CygRoot::new();

    let mut first_arg = true;
    for arg in std::env::args_os() {
        if first_arg { first_arg = false; continue; }
        let path = Path::new(&arg);
        stat_path(path, &cygroot);
    }
}
