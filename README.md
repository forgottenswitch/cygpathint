cygwin\_fs.rs
=============

Provides Win32 Rust applications with means to interpret Cygwin absolute paths and symlinks,
given that they are run from Cygwin console or script.

For usage, see the [example](example/stat.rs).
`Cargo.toml` snippet:
```
[dependencies]
cygwin_fs = "1"
```

Bugs
----
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
