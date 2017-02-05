#[cfg(windows)]
extern crate kernel32;
#[cfg(windows)]
extern crate winapi;

#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;
#[cfg(windows)]
use std::iter::once;
#[cfg(windows)]
use std::path::{Path};
#[cfg(windows)]
use std::vec::Vec;

#[cfg(windows)]
use winapi::winnt::{
    FILE_ATTRIBUTE_SYSTEM,
};

#[cfg(windows)]
pub fn maybe_cygwin_symlink(path: &Path) -> bool {
    let path_wz: Vec<u16> = path.as_os_str().encode_wide().chain(once(0)).collect();
    let attr = unsafe {
        kernel32::GetFileAttributesW(path_wz.as_ptr())
    };
    return (attr & FILE_ATTRIBUTE_SYSTEM) != 0;
}
