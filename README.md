cygwin\_symlinks.rs
===================

Provides ability for Rust on Windows applications to interpret Cygwin symlinks,
when they are run from Cygwin console or script.

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
