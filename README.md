cygwin\_fs.rs
=============

Provides Win32 Rust applications with means to interpret Cygwin absolute paths and symlinks,
given that they are run from Cygwin console or script.

This library does not do runtime linking to `cygwin1.dll`.
This is wrong, and gives all the bugs and limitations below.

For usage, see the [example](example/stat.rs).
For now, there is no crate, so clone as a git submodule, and add the following to `Cargo.toml`:
```
[dependencies]
cygwin_fs = { version = "1", path = "cygwin_fs" }
```

Bugs and limitations
--------------------
Does not support/recognize mount points.

Does not interpret the deprecated Windows Explorer Shortcut symlinks.

Relative paths are interpreted as Windows ones.

|              Symlink                 |      Cygwin     |       This crate       |
|--------------------------------------|-----------------|------------------------|
| `/symlink` points to `cygdrive/f`    |      `F:\`      | `C:\cygwin\cygdrive\f` |
| `/symlink` points to `../cygdrive/f` |      `F:\`      | `C:\cygdrive\f`        |
| `/symlink` points to `../tmp`        | `C:\cygwin\tmp` | `C:\tmp`               |

Verifying it works
------------------
In Cygwin shell:
```
ln -s /tmp /tmp/symlink
cargo run --example stat -- . C:/cygwin/tmp/symlink /tmp /cygdrive/c
```

License
-------
MIT
