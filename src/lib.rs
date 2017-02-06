#[cfg(windows)]
extern crate kernel32;
#[cfg(windows)]
extern crate winapi;

#[cfg(windows)]
use std::ffi::OsString;
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

#[cfg(windows)]
pub struct CygRoot {
    native_path_to_root: PathBuf,
    running_under_cygwin: bool,
}

#[cfg(not(windows))]
pub struct CygRoot {
}

impl CygRoot {
    #[cfg(not(windows))]
    pub fn new() -> CygRoot {
        CygRoot {}
    }

    #[cfg(not(windows))]
    pub fn running_under_cygwin(&self) -> bool {
        false
    }

    #[cfg(windows)]
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

    #[cfg(windows)]
    pub fn root_path(&self) -> &Path {
        self.native_path_to_root.as_path()
    }

    #[cfg(windows)]
    pub fn running_under_cygwin(&self) -> bool {
        self.running_under_cygwin
    }
}

#[cfg(not(windows))]
pub fn maybe_cygwin_symlink(path: &Path) -> bool {
    false
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
