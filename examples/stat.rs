extern crate cygwin_fs;

use cygwin_fs::{CygRoot, maybe_cygwin_symlink};

use std::ffi::OsStr;

fn stat_path(cygpath: &OsStr, cygroot: &CygRoot) {
    println!("{:?}", cygpath);

    if !cygroot.running_under_cygwin() {
        println!("  Not running under cygwin.");
    } else {
        println!("  Cygwin root: {:?}", cygroot.root_path());

        let winpath = cygroot.convert_path_to_native(&cygpath);
        println!("  Converted to native: {:?}", winpath);

        println!("  Maybe cygwin symlink: {}", maybe_cygwin_symlink(&winpath.as_path()));
    }
}

fn main() {
    let cygroot = CygRoot::new();

    let mut first_arg = true;
    for arg in std::env::args_os() {
        if first_arg { first_arg = false; continue; }
        stat_path(arg.as_os_str(), &cygroot);
    }
}
