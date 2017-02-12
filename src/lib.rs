/*!
Provides Win32 applications with ability to interpret,
with some quirks, Cygwin absolute paths and symlinks,
given that they are run from Cygwin shell or script.

For bugs description, see the documentation for `join_symlink_native_path_and_cygwin_target` below.
The deprecated Windows Explorer Shortcut symlinks are not interpreted.

```rust
extern crate cygwin_fs;

fn main() {
    let p = ::std::path::PathBuf::from("/cygwin/path");

    let cygroot = cygwin_fs::CygRoot::new();
    let w = cygroot.resolve_path(p.as_path());

    println!("{:?} => {:?}", p, w);
}
```

If file access time matters:
```rust,no_run
if cfg!(windows) {
    if cygwin_fs::maybe_cygwin_symlink(p) {
        ...
    }
}
```

*/

#[cfg(windows)]
extern crate kernel32;
#[cfg(windows)]
extern crate winapi;

use std::path::{Path,PathBuf};

#[cfg(windows)]
use std::ffi::{OsString};
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;
#[cfg(windows)]
use std::iter::once;
#[cfg(windows)]
use std::vec::Vec;
#[cfg(windows)]
use std::fs::File;
#[cfg(windows)]
use std::io::Read;

#[cfg(windows)]
use winapi::winnt::{
    FILE_ATTRIBUTE_SYSTEM,
};

// Stub

#[cfg(not(windows))]
#[derive(Clone,Debug)]
pub struct CygRoot {
    empty_root_pathbuf: PathBuf,
}

#[cfg(not(windows))]
impl CygRoot {
    pub fn new() -> CygRoot {
        CygRoot {
            empty_root_pathbuf: PathBuf::new(),
        }
    }
    pub fn from(_native_path_to_root: PathBuf, _under_cygwin: bool) -> CygRoot {
        CygRoot {
            empty_root_pathbuf: PathBuf::new(),
        }
    }

    pub fn root_path(&self) -> &Path { self.empty_root_pathbuf.as_path() }
    pub fn running_under_cygwin(&self) -> bool { false }
    pub fn convert_path_to_native(&self, _path: &str) -> PathBuf { PathBuf::new() }
    pub fn read_symlink_contents(&self, _path: &Path) -> Option<PathBuf> { None }
    pub fn resolve_symlink_once(&self, _path: &Path) -> PathBuf { PathBuf::new() }
    pub fn resolve_symlink(&self, _path: &Path) -> PathBuf { PathBuf::new() }

    pub fn join_symlink_native_path_and_cygwin_target(&self, _native_path: &Path,
            _cygwin_path: &Path) -> PathBuf { PathBuf::new() }
    pub fn resolve_path(&self, path: &Path) -> PathBuf { PathBuf::from(path) }
}

#[cfg(not(windows))]
pub fn maybe_cygwin_symlink(_path: &Path) -> bool { false }

// Implementation

/// An object that remembers the current Cygwin root path,
/// for use in path resolving operations.
#[cfg(windows)]
#[derive(Clone,Debug)]
pub struct CygRoot {
    native_path_to_root: PathBuf,
    running_under_cygwin: bool,
}

#[cfg(windows)]
impl CygRoot {
    /// Looks up cygwin1.dll in PATH, and marks the path two dirs upper as a Cygwin root.
    /// This is because Cygwin keeps the dll in /bin.
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

    /// Constructs an arbitrary CygRoot.
    pub fn from(native_path_to_root: PathBuf, under_cygwin: bool) -> CygRoot {
        CygRoot {
            running_under_cygwin: under_cygwin,
            native_path_to_root: native_path_to_root,
        }
    }

    /// Returns Windows path to Cygwin root.
    /// This involves no processing; everything is done in CygRoot::new().
    /// Should only be called with cfg!(windows).
    pub fn root_path(&self) -> &Path {
        self.native_path_to_root.as_path()
    }

    /// Whether running in a Cygwin shell.
    /// This involves no check; everything is done in CygRoot::new().
    pub fn running_under_cygwin(&self) -> bool {
        self.running_under_cygwin
    }

    /// Converts /cygwin/path to C:\native\one, without following symlinks.
    /// Should only be called if self.running_under_cygwin() returns true.
    pub fn convert_path_to_native(&self, path: &str) -> PathBuf {
        let path_b = path.as_bytes();
        let mut ret = PathBuf::new();
        let mut cygdrive_end = 0;
        if path.starts_with("/") {
            let mut cygdrive = '\0';
            let path_blen = path_b.len();
            // Check for /cygdrive/drive_letter/
            if let Some(path_1) = eat_chars(path, '/') {
                if let Some(path_11) = eat_str(path_1, "cygdrive") {
                    if let Some(path_2) = eat_chars(path_11, '/') {
                        if let Some((drive_letter, path_22)) = pop_char(path_2) {
                            if valid_drive_letter(drive_letter) {
                                if path_22.len() == 0 {
                                    cygdrive = ascii_upcase(drive_letter);
                                    cygdrive_end = path_blen;
                                } else {
                                    if let Some(path_3) = eat_chars(path_22, '/') {
                                        cygdrive = ascii_upcase(drive_letter);
                                        cygdrive_end = path_blen - path_3.as_bytes().len();
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if cygdrive != '\0' {
                ret.push(format!("{}:\\", cygdrive));
            } else {
                ret.push(&self.native_path_to_root);
            }
        }
        let mut last_was_slash = false;
        let mut beg_path_comp = 0;
        let mut just_past_cygdrive = false;
        for (i, ch) in path.char_indices() {
            if i < cygdrive_end {
                just_past_cygdrive = true;
                beg_path_comp = i;
                continue;
            }
            if just_past_cygdrive {
                just_past_cygdrive = false;
                last_was_slash = true;
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
                beg_path_comp = i;
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

    /// Retrieves contents of a C:\cygwin\symlink file
    /// Should only be called if both self.running_under_cygwin() and maybe_cygwin_symlink(path) return true.
    pub fn read_symlink_contents(&self, path: &Path) -> Option<PathBuf> {
        let mut fdata = Vec::<u8>::with_capacity(64);
        match File::open(path) {
            Err(_) => return None,
            Ok(mut f) => {
                match f.read_to_end(&mut fdata) {
                    Err(_) => return None,
                    Ok(_) => {
                        let filemagic = b"!<symlink>";
                        if !fdata.as_slice().starts_with(filemagic) {
                            return None
                        } else {
                            let data_after_magic = &fdata[filemagic.len()..];
                            let string16_in_file = string_from_utf_bom_lossy(data_after_magic);
                            let path16_in_file = PathBuf::from(&string16_in_file);
                            return Some(path16_in_file)
                        }
                    }
                }
            }
        }
    }

    /// Follows C:\cygwin\symlink once, returning C:\cygwin\target
    /// If path to the cygwin symlink is relative, return value is relative too.
    /// Should only be called if both self.running_under_cygwin() and maybe_cygwin_symlink(path) return true.
    pub fn resolve_symlink_once(&self, path: &Path) -> PathBuf {
        match self.read_symlink_contents(path) {
            None => return PathBuf::from(path),
            Some(cygwin_target) => {
                return self.join_symlink_native_path_and_cygwin_target(path, &cygwin_target.as_path())
            }
        }
    }

    /// Follows C:\cygwin\symlink as many times as needed, returning C:\cygwin\target
    /// If path to the cygwin symlink is relative, return value is relative too
    /// (unless a further symlink points to an absolute path).
    /// Should only be called if both self.running_under_cygwin() and maybe_cygwin_symlink(path) return true.
    pub fn resolve_symlink(&self, path: &Path) -> PathBuf {
        let mut dest = PathBuf::from(path);
        let mut first_iteration = true;
        loop {
            if first_iteration {
                first_iteration = false
            } else {
                if !maybe_cygwin_symlink(&dest.as_path()) {
                    return dest
                }
            }
            match self.read_symlink_contents(&dest.as_path()) {
                None => return dest,
                Some(cygwin_target) => {
                    dest = self.join_symlink_native_path_and_cygwin_target(path, &cygwin_target.as_path())
                }
            }
        }
    }

    /// Concatenates C:\cygwin\dir1\symlink with:
    /// - dir2/target into C:\cygwin\dir1\dir2\target
    /// - /dir2/target into C:\cygwin\dir2\target
    /// - /cygdrive/d/dir2/target into D:\dir2\target
    /// Bugs:
    /// - ../../../../target into C:\target, not C:\cygwin\target
    /// - ../../../../cygdrive/d into C:\cygdrive\d, not D:\
    ///
    /// Should only be called with cfg!(windows).
    pub fn join_symlink_native_path_and_cygwin_target(&self, native_path: &Path, cygwin_path: &Path) -> PathBuf {
        let mut cygwin_path_s = cygwin_path.as_os_str().to_string_lossy().into_owned();
        if cygwin_path.starts_with("/") {
            return self.convert_path_to_native(&cygwin_path_s.as_str());
        } else {
            backslash_the_slashes_in_string(&mut cygwin_path_s);
            match native_path.parent() {
                None => PathBuf::from(cygwin_path_s),
                Some(dir) => dir.join(cygwin_path_s),
            }
        }
    }

    /// Converts /cygwin/path to C:\native\one, following Cygwin symlinks.
    /// Return value could be relative, as in `resolve_symlink`.
    /// Could be called without being wrapped in any checks (unlike other methods), even not on cfg!(windows).
    pub fn resolve_path(&self, p: &Path) -> PathBuf {
        if !self.running_under_cygwin { return PathBuf::from(p) }
        let p_native =
            if p.starts_with("/") {
                let p_s = p.as_os_str().to_string_lossy().into_owned();
                self.convert_path_to_native(&p_s.as_str())
            } else {
                PathBuf::from(p)
            };
        if !maybe_cygwin_symlink(&p_native.as_path()) { return p_native }
        return self.resolve_symlink(&p_native.as_path())
    }
}

/// Queries the file system about whether the file could be a Cygwin symlink.
/// Always false not on cfg!(windows).
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

#[cfg(windows)]
fn string_from_utf_bom_lossy(data: &[u8]) -> String {
    let byte_order_mark = &data[..2];
    let byte_order_mark_islen = byte_order_mark.len() == 2;
    let data16_is_big_endian =
        byte_order_mark_islen && byte_order_mark[0] == 254 && byte_order_mark[1] == 255;
    let data16_is_little_endian =
        byte_order_mark_islen && byte_order_mark[0] == 255 && byte_order_mark[1] == 254;

    if !data16_is_big_endian && !data16_is_little_endian {
        return String::from_utf8_lossy(data).into_owned();
    } else {
        let data_nobom16 = &data[2..];
        let mut codepoints_in_data = Vec::<u16>::with_capacity(data_nobom16.len() / 2);
        let mut even_byte = true;
        let mut codepoint: u16 = 0;
        for (i, b) in data_nobom16.iter().enumerate() {
            let upper_byte =
                if data16_is_big_endian { even_byte } else { !even_byte };
            let b16 = *b as u16;
            codepoint +=
                if upper_byte { b16 * 256 } else { b16 };
            if !even_byte {
                if codepoint == 0 && i != 0 { break }
                codepoints_in_data.push(codepoint);
                codepoint = 0;
            }
            even_byte = !even_byte;
        }
        return String::from_utf16_lossy(codepoints_in_data.as_slice());
    }
}

#[cfg(windows)]
fn backslash_the_slashes_in_string(s: &mut String) {
    unsafe {
        let v = s.as_mut_vec();
        for mut b in v.iter_mut() {
            if *b == b'/' {
                *b = b'\\';
            }
        }
    }
}

#[cfg(test)]
#[cfg(windows)]
mod win32_tests {

use std::path::PathBuf;

use CygRoot;
use string_from_utf_bom_lossy;

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
    let posix = String::from("/tmp");
    let win32_p = cygroot.convert_path_to_native(&posix.as_str());
    let win32_s = win32_p.as_os_str().to_string_lossy().into_owned();
    assert_eq!(win32_s, "F:\\cygwin\\tmp");
}

#[test]
fn converts_absolute_posix_paths_several_levels_deep() {
    let cygroot = cygwin();
    let posix = String::from("/tmp/abc/def/ghi");
    let win32_p = cygroot.convert_path_to_native(&posix.as_str());
    let win32_s = win32_p.as_os_str().to_string_lossy().into_owned();
    assert_eq!(win32_s, "F:\\cygwin\\tmp\\abc\\def\\ghi");
}

#[test]
fn converts_absolute_posix_dirs_several_levels_deep() {
    let cygroot = cygwin();
    let posix = String::from("/tmp/abc/def/ghi/");
    let win32_p = cygroot.convert_path_to_native(&posix.as_str());
    let win32_s = win32_p.as_os_str().to_string_lossy().into_owned();
    assert_eq!(win32_s, "F:\\cygwin\\tmp\\abc\\def\\ghi");
}

#[test]
fn converts_absolute_cygdrive_paths() {
    let cygroot = cygwin();
    let posix = String::from("/cygdrive/f");
    let win32_p = cygroot.convert_path_to_native(&posix.as_str());
    let win32_s = win32_p.as_os_str().to_string_lossy().into_owned();
    assert_eq!(win32_s, "F:\\");
}

#[test]
fn converts_absolute_cygdrive_paths_several_levels_deep() {
    let cygroot = cygwin();
    let posix = String::from("/cygdrive/f/a/bb/ccc");
    let win32_p = cygroot.convert_path_to_native(&posix.as_str());
    let win32_s = win32_p.as_os_str().to_string_lossy().into_owned();
    assert_eq!(win32_s, "F:\\a\\bb\\ccc");
}

#[test]
fn converts_absolute_cygdrive_dirs_several_levels_deep() {
    let cygroot = cygwin();
    let posix = String::from("/cygdrive/f/a/bb/ccc/");
    let win32_p = cygroot.convert_path_to_native(&posix.as_str());
    let win32_s = win32_p.as_os_str().to_string_lossy().into_owned();
    assert_eq!(win32_s, "F:\\a\\bb\\ccc");
}

#[test]
fn reads_utf16le() {
    let data : Vec<u8> = vec![ 0xff, 0xfe, b'a', 0, b'b', 0 ];
    let s = string_from_utf_bom_lossy(data.as_slice());
    assert_eq!(s, "ab");
}

#[test]
fn reads_utf16be() {
    let data : Vec<u8> = vec![ 0xfe, 0xff, 0, b'a', 0, b'b' ];
    let s = string_from_utf_bom_lossy(data.as_slice());
    assert_eq!(s, "ab");
}

#[test]
fn reads_utf8_when_no_utf16bom() {
    let data : Vec<u8> = vec![ b'a', b'b' ];
    let s = string_from_utf_bom_lossy(data.as_slice());
    assert_eq!(s, "ab");
}

#[test]
fn joins_symlink_native_path_and_cygwin_relative_target() {
    let cygroot = cygwin();
    let symlink = PathBuf::from("C:\\dir1\\symlink");
    let target = PathBuf::from("a/bb/ccc");
    let win32_p = cygroot.join_symlink_native_path_and_cygwin_target(symlink.as_path(), target.as_path());
    let win32_s = win32_p.as_os_str().to_string_lossy().into_owned();
    assert_eq!(win32_s, "C:\\dir1\\a\\bb\\ccc");
}

#[test]
fn joins_symlink_native_path_and_cygwin_absolute_target() {
    let cygroot = cygwin();
    let symlink = PathBuf::from("C:\\dir1\\symlink");
    let target = PathBuf::from("/a/bb/ccc");
    let win32_p = cygroot.join_symlink_native_path_and_cygwin_target(symlink.as_path(), target.as_path());
    let win32_s = win32_p.as_os_str().to_string_lossy().into_owned();
    assert_eq!(win32_s, "F:\\cygwin\\a\\bb\\ccc");
}

#[test]
fn joins_symlink_native_path_and_cygwin_absolute_cygdrive_target() {
    let cygroot = cygwin();
    let symlink = PathBuf::from("C:\\dir1\\symlink");
    let target = PathBuf::from("/cygdrive/a/bb/ccc");
    let win32_p = cygroot.join_symlink_native_path_and_cygwin_target(symlink.as_path(), target.as_path());
    let win32_s = win32_p.as_os_str().to_string_lossy().into_owned();
    assert_eq!(win32_s, "A:\\bb\\ccc");
}

#[test]
fn joins_symlink_empty_native_path_and_cygwin_absolute_target() {
    let cygroot = cygwin();
    let symlink = PathBuf::from("");
    let target = PathBuf::from("/a/bb/ccc");
    let win32_p = cygroot.join_symlink_native_path_and_cygwin_target(symlink.as_path(), target.as_path());
    let win32_s = win32_p.as_os_str().to_string_lossy().into_owned();
    assert_eq!(win32_s, "F:\\cygwin\\a\\bb\\ccc");
}

#[test]
fn joins_symlink_empty_native_path_and_cygwin_relative_target() {
    let cygroot = cygwin();
    let symlink = PathBuf::from("");
    let target = PathBuf::from("a/bb/ccc");
    let win32_p = cygroot.join_symlink_native_path_and_cygwin_target(symlink.as_path(), target.as_path());
    let win32_s = win32_p.as_os_str().to_string_lossy().into_owned();
    assert_eq!(win32_s, "a\\bb\\ccc");
}

}
