# shared-library-builder-rs

Extendable cross-platform shared library builder. The library is designed to create statically linked libraries and supports Rust and CMake libraries out of the box. However, it allows developers to plug-in `makefile` based libraries by implementing a `Library` trait.