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
        let mut cygdrive_end = 0;
        if path_s.as_str().starts_with("/") {
            let mut cygdrive = '\0';
            let path_blen = path_b.len();
            // Check for /cygdrive/drive_letter/
            eat_chars(path_s.as_str(), '/').and_then(|path_1| {
                eat_str(path_1, "cygdrive").and_then(|path_11| {
                    eat_chars(path_11, '/').and_then(|path_2| {
                        pop_char(path_2).and_then(|(drive_letter, path_22)| {
                            if valid_drive_letter(drive_letter) {
                                if path_22.len() == 0 {
                                    cygdrive = ascii_upcase(drive_letter);
                                    cygdrive_end = path_blen;
                                } else {
                                    eat_chars(path_22, '/').and_then(|path_3| {
                                        cygdrive = ascii_upcase(drive_letter);
                                        cygdrive_end = path_blen - path_3.as_bytes().len();
                                        Some(0)
                                    });
                                };
                            };
                            Some(0)
                        });
                        Some(0)
                    });
                    Some(0)
                });
                Some(0)
            });
            if cygdrive != '\0' {
                ret.push(format!("{}:\\", cygdrive));
            } else {
                ret.push(&self.native_path_to_root);
            }
        }
        let mut last_was_slash = false;
        let mut beg_path_comp = 0;
        let mut just_past_cygdrive = false;
        for (i, ch) in path_s.char_indices() {
            if i < cygdrive_end {
                just_past_cygdrive = true;
                beg_path_comp = i;
                continue;
            }
            if just_past_cygdrive {
                just_past_cygdrive = false;
                beg_path_comp = i;
            }
            if ch == '/' || ch == '\\' {
                if !last_was_slash {
                    if i > beg_path_comp {
                        let path_component = unsafe {
                            ::std::str::from_utf8_unchecked(&path_b[beg_path_comp..i])
                        };
                        ret.push(path_component);
                    }
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
            ret.push(final_path_component);
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

#[cfg(windows)]
fn valid_drive_letter(x: char) -> bool {
    (x >= 'a' && x <= 'z') || (x >= 'A' && x <= 'Z')
}

#[cfg(windows)]
fn pop_char<'a>(s: &'a str) -> Option<(char, &'a str)> {
    let mut ret_beg = 0;
    let mut ret_ch = None;
    for (i, ch) in s.char_indices() {
        if i == 0 {
            ret_ch = Some(ch);
        } else {
            ret_beg = i;
            break;
        }
    }
    match ret_ch {
        None => None,
        Some(ch) => {
            if ret_beg == 0 {
                Some((ch, ""))
            } else {
                let ret_b = &s.as_bytes()[ret_beg..];
                let ret_s = unsafe {
                    ::std::str::from_utf8_unchecked(ret_b)
                };
                Some((ch, ret_s))
            }
        }
    }
}

#[cfg(windows)]
fn eat_chars<'a>(s: &'a str, x: char) -> Option<&'a str> {
    let mut ret_beg = 0;
    for (i, ch) in s.char_indices() {
        if ch != x {
            ret_beg = i;
            break;
        }
    }
    if ret_beg == 0 {
        None
    } else {
        let ret_b = &s.as_bytes()[ret_beg..];
        let ret_s = unsafe {
            ::std::str::from_utf8_unchecked(ret_b)
        };
        Some(ret_s)
    }
}

#[cfg(windows)]
fn eat_str<'a>(s: &'a str, s1: &str) -> Option<&'a str> {
    let mut chs1 = s1.chars();
    let mut ret_beg = 0;
    for (i, ch) in s.char_indices() {
        match chs1.next() {
            None => {
                ret_beg = i;
                break;
            },
            Some(x) => {
                if ch != x {
                    return None;
                }
            },
        }
    }
    if chs1.next().is_some() {
        None
    } else {
        let ret_b = &s.as_bytes()[ret_beg..];
        let ret_s = unsafe {
            ::std::str::from_utf8_unchecked(ret_b)
        };
        Some(ret_s)
    }
}

#[cfg(windows)]
fn ascii_upcase(x: char) -> char {
    if x >= 'a' && x <= 'z' {
        let c = x as u32;
        let d = c - ('a' as u32);
        return ::std::char::from_u32(('A' as u32) + d).unwrap_or('\0');
    }
    return x;
}

#[cfg(test)]
#[cfg(windows)]
mod win32_tests {

use std::ffi::OsString;
use std::path::PathBuf;

use CygRoot;

fn cygwin() -> CygRoot {
    let root = PathBuf::from("F:\\cygwin");
    return CygRoot {
        running_under_cygwin: true,
        native_path_to_root: root,
    };
}

#[test]
fn converts_absolute_posix_paths() {
    let cygroot = cygwin();
    let posix = OsString::from("/tmp");
    let win32_p = cygroot.convert_path_to_native(posix.as_os_str());
    let win32_s = win32_p.as_os_str().to_string_lossy().into_owned();
    assert_eq!(win32_s, "F:\\cygwin\\tmp");
}

#[test]
fn converts_absolute_posix_paths_several_levels_deep() {
    let cygroot = cygwin();
    let posix = OsString::from("/tmp/abc/def/ghi");
    let win32_p = cygroot.convert_path_to_native(posix.as_os_str());
    let win32_s = win32_p.as_os_str().to_string_lossy().into_owned();
    assert_eq!(win32_s, "F:\\cygwin\\tmp\\abc\\def\\ghi");
}

#[test]
fn converts_absolute_cygdrive_paths() {
    let cygroot = cygwin();
    let posix = OsString::from("/cygdrive/f");
    let win32_p = cygroot.convert_path_to_native(posix.as_os_str());
    let win32_s = win32_p.as_os_str().to_string_lossy().into_owned();
    assert_eq!(win32_s, "F:\\");
}

#[test]
fn converts_absolute_cygdrive_paths_several_levels_deep() {
    let cygroot = cygwin();
    let posix = OsString::from("/cygdrive/f/a/bb/ccc");
    let win32_p = cygroot.convert_path_to_native(posix.as_os_str());
    let win32_s = win32_p.as_os_str().to_string_lossy().into_owned();
    assert_eq!(win32_s, "F:\\a\\bb\\ccc");
}

}
