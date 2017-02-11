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

        let maybe_cyglink = maybe_cygwin_symlink(&winpath.as_path());
        println!("  Maybe cygwin symlink: {}", maybe_cyglink);

        if maybe_cyglink {
            let link_txt = cygroot.read_symlink_contents(&winpath.as_path());
            println!("    Symlink contents: {:?}", link_txt);

            let link_dest1 = cygroot.resolve_symlink_once(&winpath.as_path());
            println!("    Symlink's first destination: {:?}", link_dest1);

            let link_dest = cygroot.resolve_symlink(&winpath.as_path());
            println!("    Symlink's final destination: {:?}", link_dest);
        }
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
