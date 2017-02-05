extern crate cygwin_path;

use std::path::Path;

#[cfg(not(windows))]
fn stat_path(path: &Path) {
    println!("{:?}", path);
}

#[cfg(windows)]
fn stat_path(path: &Path) {
    println!("{:?}", path);

    let maybe_cygwin_symlink = cygwin_path::maybe_cygwin_symlink(&path);
    println!("  maybe cygwin symlink: {}", maybe_cygwin_symlink);
}

fn main() {
    let mut first_arg = true;
    for arg in std::env::args_os() {
        if first_arg { first_arg = false; continue; }
        let path = Path::new(&arg);
        stat_path(path);
    }
}
