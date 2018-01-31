LiftInstall
===========

An installer for your application. Designed to be customisable to the core, hookable from external
 applications, and have a decent UI.

This is designed to be a more modern interpretation of Qt's Installer Framework, which has several issues:
- Hard to develop on and poorly documented
- Hardcoded package listing format, requires very specific setups for packages, packages must be built
    using their tool
- Poorly supported, with rare updates and a large list of bugs

Building
--------

Add your logo to `static/img/logo.png`, modify the configuration file, then run:

```bash
cargo build
```

License
-------

LiftInstall is licensed under the Apache 2.0 License, which can be found in [LICENSE](LICENSE).
