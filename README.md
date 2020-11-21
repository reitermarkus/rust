# A fork of the Rust compiler for ESP, with STD support

This fork enables projects to be built for the ESP32 and ESP8266 using [espressif's llvm fork](https://github.com/espressif/llvm-project).
Moreover, the fork contains a port of [Rust](https://github.com/rust-lang/rust)'s STD lib on top of the ESP-IDF libraries.

The repo is essentially a (more recent) fork of the original work of [MabezDev](https://github.com/MabezDev/rust-xtensa) which enables support in Rust for the Xtensa/ESP8266/ESP32 targets.
... plus a set of extensions of the std library so that it can compile and run on top of ESP-IDF.

## Background

A few words on the approach taken for bringing STD support to the ESP platform:
* Even though the ESPs are considered "bare metal", the ESP-IDF framework of Espressif is actually a relatively complete Posix layer
* If you squint a little, you can even pretend that an ESP running the ESP-IDF framework - from the point of view of the app on top - looks like a real Unix. That's because ESP-IDF has a libc (newlib), BSD sockets layer (lwip), pthread support (running on top of FreeRTOS) and quite a few other unix/posix APIs. It also uses GCC, elf and does have a port of LLVM (as per above)
* So besides the process and env APIs which are obviously not available on the ESP, everything else looks like regular unix/posix C API layer

Therefore, the STD support for ESP is implemented inside Rust's standard std::sys::unix modules and more or less boils down to stubbing out functionality which is not available in ESP-IDF. The whole ESP STD patch-set is having currently the miniscule size of ~500-1000 LOCs.

## Disclaimer

STD for ESP **does** require the ESP-IDF toolkit and **does** call into ESP-IDF. This is contrary to [other efforts (esp-rs) related to running Rust on ESP](https://github.com/esp-rs), which are trying to avoid any dependencies on the vendor-provided software stack.

## Forum

Rust on ESP seems to be discussed here: https://matrix.to/#/#esp-rs:matrix.org!

## "Hello, World" demo app

[Here](https://github.com/ivmarkov/rust-esp32-std-hello)

## Building

Install [Rustup](https://rustup.rs/).

Build using these steps (NOTE: building might take **close to an hour**!):
```sh
$ cd <some directory where you'll keep the compiler binaries and its sources; you'll need to keep the whole GIT repo, because xargo/cargo need those when building your ESP32 crates>
$ git clone https://github.com/ivmarkov/rust
$ cd rust
$ ./configure --experimental-targets=Xtensa
$ ./x.py build --stage 2
```

Make Rustup aware of the newly built compiler:

```sh
$ rustup toolchain link xtensa ~/<...>/rust/build/x86_64-unknown-linux-gnu/stage2
```

Switch to the new compiler in Rustup:

```sh
$ rustup default xtensa
```

Check the compiler:
```sh
$ rustc --print target-list
```

At the end of the printed list of targets you should see:
```
...
xtensa-esp32-none-elf
xtensa-esp8266-none-elf
xtensa-none-elf
```

### Optional steps

Install xargo (you'll need it when building your ESP32 crates, because `cargo -Z build-std` corrently does not seem to work):

```sh
$ cargo install xargo
```

Set xargo's XARGO_RUST_SRC dir:

```sh
export XARGO_RUST_SRC=`rustc --print sysroot`/lib/rustlib/src/rust/library
```

... and probably put that line at the end of `~/.profile`, `~/.bash_profile` or `~/.bashrc` but make sure it is executed **after** the modification of $PATH with `~/.cargo/bin` by Rustup
 
## Updating this fork

TBD: https://github.com/ivmarkov/rust-xtensa-patches
