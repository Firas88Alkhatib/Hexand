# Hexand

Minimal Operating System Kernel written in Rust.

## Usage

1. Download and install [QEMU](https://www.qemu.org/)
2. Download and install [Rust](https://www.rust-lang.org/tools/install)
3. Install Rust nightly: `rustup install nightly`
4. Set Rust Nightly as the default compiler for Hexand Project by navigating to Hexand directory and run: `rustup override set nightly`
5. Add Rust llvm tools component: `rustup component add llvm-tools-preview`
6. Finally run: `cargo run`

<br>

\
*This project is inspired by [Philipp Oppermann](https://github.com/phil-opp) and his tutorial about writing an operating system using Rust https://os.phil-opp.com *.
