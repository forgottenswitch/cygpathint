cygwin\_fs.rs
===================

Provides means for Rust on Windows applications to interpret Cygwin absolute paths and symlinks,
given that they are run from Cygwin console or script.

Testing
-------
In Cygwin shell:
```
ln -s /tmp /tmp/symlink
cargo run --example stat -- . C:/cygwin/tmp/symlink
```

License
-------
MIT
