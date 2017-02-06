#[cfg(windows)]
extern crate kernel32;
#[cfg(windows)]
extern crate winapi;

#[cfg(windows)]
use std::ffi::{OsStr,OsString};
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;
#[cfg(windows)]
use std::iter::once;
#[cfg(windows)]
use std::path::{Path,PathBuf};
#[cfg(windows)]
use std::vec::Vec;

#[cfg(windows)]
use winapi::winnt::{
    FILE_ATTRIBUTE_SYSTEM,
};

// Stub

#[cfg(not(windows))]
pub struct CygRoot {
}

#[cfg(not(windows))]
impl CygRoot {
    pub fn new() -> CygRoot {
        CygRoot {}
    }

    pub fn running_under_cygwin(&self) -> bool {
        false
    }
}

#[cfg(not(windows))]
pub fn maybe_cygwin_symlink(path: &Path) -> bool {
    false
}

// Implementation

#[cfg(windows)]
pub struct CygRoot {
    native_path_to_root: PathBuf,
    running_under_cygwin: bool,
}

#[cfg(windows)]
impl CygRoot {
    pub fn new() -> CygRoot {
        let env_path = std::env::var_os("PATH");
        let cygwin_dll_name = Path::new("cygwin1.dll");
        let mut under_cygwin = false;
        let root =
            match find_in_pathlist(&env_path, &cygwin_dll_name) {
                None => PathBuf::new(),
                Some(cygwin_dll_path) =>
                    match cygwin_dll_path.parent() {
                        None => PathBuf::new(),
                        Some(bin_path) =>
                            match bin_path.parent() {
                                None => PathBuf::new(),
                                Some(root_path) => {
                                    under_cygwin = true;
                                    PathBuf::new().join(root_path)
                                }
                            },
                    },
            };
        CygRoot {
            running_under_cygwin: under_cygwin,
            native_path_to_root: root,
        }
    }

    pub fn root_path(&self) -> &Path {
        self.native_path_to_root.as_path()
    }

    pub fn running_under_cygwin(&self) -> bool {
        self.running_under_cygwin
    }

    /// Converts /cygwin/path to C:\native\one, without following symlinks.
    pub fn convert_path_to_native(&self, path: &OsStr) -> PathBuf {
        let path_s = path.to_string_lossy().into_owned();
        let path_b = path_s.as_bytes();
        let mut ret = PathBuf::new();
        if path_s.as_str().starts_with("/") {
            ret.push(&self.native_path_to_root);
        }
        let mut last_was_slash = false;
        let mut beg_path_comp = 0;
        for (i, ch) in path_s.char_indices() {
            if ch == '/' || ch == '\\' {
                if !last_was_slash {
                    if i > beg_path_comp {
                        let path_component = unsafe {
                            ::std::str::from_utf8_unchecked(&path_b[beg_path_comp..i])
                        };
                        ret.push(path_component);
                    }
                    ret.join("\\");
                    last_was_slash = true;
                }
                continue;
            }
            if last_was_slash {
                last_was_slash = false;
                beg_path_comp = i;
            }
        }
        if beg_path_comp < path_b.len() - 1 {
            let final_path_component = unsafe {
                ::std::str::from_utf8_unchecked(&path_b[beg_path_comp..])
            };
            ret.join(final_path_component);
        }
        ret
    }
}

#[cfg(windows)]
pub fn maybe_cygwin_symlink(path: &Path) -> bool {
    let path_wz: Vec<u16> = path.as_os_str().encode_wide().chain(once(0)).collect();
    let attr = unsafe {
        kernel32::GetFileAttributesW(path_wz.as_ptr())
    };
    if attr == winapi::INVALID_FILE_ATTRIBUTES {
        return false;
    }
    return (attr & FILE_ATTRIBUTE_SYSTEM) != 0;
}

// Utilites

#[cfg(windows)]
fn find_in_pathlist(pathlist: &Option<OsString>, filename: &Path) -> Option<PathBuf> {
    match pathlist {
        &None => None,
        &Some(ref pathlist_os) => {
            for dir in std::env::split_paths(&pathlist_os) {
                let filepath = dir.join(filename);
                if filepath.is_file() {
                    return Some(filepath);
                }
            }
            None
        }
    }
}
