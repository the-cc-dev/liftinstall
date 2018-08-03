LiftInstall
===========

[![Build Status](https://travis-ci.org/j-selby/liftinstall.svg?branch=master)](https://travis-ci.org/j-selby/liftinstall)

- Usage Documentation: https://liftinstall.jselby.net/

An installer for your application. Designed to be customisable to the core, hookable from external
 applications, and have a decent UI.

This is designed to be a more modern interpretation of Qt's Installer Framework, which has several issues:
- Hard to develop on and poorly documented
- Hardcoded package listing format, requires very specific setups for packages, packages must be built
    using their tool
- Poorly supported, with rare updates and a large list of bugs

Building
--------

- Add your favicon to `static/favicon.ico`
- Modify the configuration file as needed
- Tweak `package.metadata.winres` metadata in `Cargo.toml`
- Run:

```bash
cargo build --release
```

LiftInstall should build on both Stable and Nightly Rust.

Contributing
------------

PRs are very welcome. Code should be run through [Rustfmt](https://github.com/rust-lang-nursery/rustfmt) 
 before submission.

License
-------

LiftInstall is licensed under the Apache 2.0 License, which can be found in [LICENSE](LICENSE).
