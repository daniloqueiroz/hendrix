# About

Hendrix is a wannabe experimental Microkernel for x86_64 written in Rust.

The Kernel itself takes care of scheduling, memory management, I/O and IPC. 



# Development

## Before start

This project follows the [Writing an OS in Rust](https://os.phil-opp.com/) blog.

To be able to build the project, you need `Rust nightly`.
There's also a few other tooling we need, for instance
we need to compile `rust core` using the kernel toolchain.
Install the following to be able to compile **hendrix**:
 
```bash
rustup override set nightly
rustup component add rust-src
rustup component add llvm-tools-preview
cargo install bootimage
```

## Building the project

To run **Hendrix** you need to have [QEMU](https://www.qemu.org/) installed.

Do a `cargo run` to launch the kernel or `cargo test` to run all the tests.

