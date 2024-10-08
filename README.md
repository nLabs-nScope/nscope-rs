# nscope-rs
[![Crates.io](https://img.shields.io/crates/v/nscope)](https://crates.io/crates/nscope)
[![License](https://img.shields.io/crates/l/nscope)](LICENSE)
[![Documentation](https://docs.rs/nscope/badge.svg)](https://docs.rs/nscope)

![Cargo Test](https://github.com/nLabs-nScope/nscope-rs/actions/workflows/tests.yml/badge.svg)

A Rust implementation of the nScope API.

## Building from Source

The nScope API can be built and run from source to enable users and developers to quickly iterate on nScope source code. To establish a development environment, follow the steps below.


### Prerequisites

1. Rust Toolchain (https://rustup.rs)

After installing the development dependencies, check to make sure you have a working environment by running version commands for each of the required tools.

```shell
$ rustup --version
```
The above commands should print a version successfully.

> **Note** - macOS specifics
>
> On macOS the project is configured to build a universal binary, including both x86 and Apple Silicon binaries in one. To enable that, we must add both rust target toolchains as follows:
> ```shell
> rustup target add x86_64-apple-darwin
> rustup target add aarch64-apple-darwin
> ```

> **Note** - Linux specifics
>
> On linux distributions, we need the system library headers for `libusb` and `libudev`. To install these on an Ubuntu distribtion, the following command should work.
> ```shell
> sudo apt-get install libusb-1.0-0-dev libudev-dev
> ```
> On other distributions, developers should look to their package managers for these development headers.

### Clone and Install Development Dependencies

```shell
$ git clone https://github.com/nLabs-nScope/nscope-rs.git
$ cd nscope-rs
```

### Build and Run
```shell
$ cargo build
$ cargo run --example list_all_nscopes
```

## Python Development

This project also supports a python interface to the nScope. To set up an environment for python development, follow the steps below.

### Prerequisites

1. All steps above
2. A Python 3.9 or newer installation

### Create a virtual environment and activate it
```shell
$ python3 -m venv venv
$ source venv/bin/activate
```

### Install python dependencies
```shell
$ pip install maturin
```

### Build and install the nscope package for developement
```shell
$ maturin develop
```

### Build and install the nscope package for developement
```shell
$ maturin develop
$ python examples/list_all_nscopes.py 
```
