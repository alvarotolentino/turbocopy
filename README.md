# Turbo Copy in Rust

This is a Rust project that provides different methods for copying files. It includes implementations using both the standard libc and the io_uring library.

## Getting Started

These instructions will get you a copy of the project up and running on your local machine for development and testing purposes.

### Prerequisites

You need to have Rust installed on your machine. Follow the instructions on the [official Rust website](https://www.rust-lang.org/tools/install) to install Rust.

### Installing

Clone the repository and build the project:

```bash
git clone https://github.com/alvarotolentino/turbocopy.git
cd turbocopy
cargo build
```

# Usage

This library provides the TurboCopy trait with a copy_with method. Two structs, CopyLibc and CopyIoUring, implement this trait using different methods for copying files.

```rust
let source_path = PathBuf::from("path/to/source");
let target_path = PathBuf::from("path/to/target");
let blocksize = 1024;

// standard libc implementation
let copier = CopyLibc;
copier.copy_with(&source_path, &target_path, blocksize);

// io_uring implementation
let copier = CopyIoUring;
copier.copy_with(&source_path, &target_path, blocksize);

```

If you want to run as a command in terminal

Usage: copy [OPTIONS] <COMMAND>

Commands:
  synchronous      Use read and write syscalls
  io-uring  Use uring with SQPOLL
  help      Print this message or the help of the given subcommand(s)

Options:
  -p, --path <PATH>             The path where source will be found and target will be written to. Defaults to the current directory.
  -f, --filename <FILENAME>     The name of the file to be written to. Defaults to 'chunck'.
  -b, --blocksize <BLOCK_SIZE>  Blocksize used defaults to filesystem's fstat report multiplied by 32.
  -h, --help                    Print help
  -V, --version                 Print version

Below the result of copy a 1GB file

`CopyLibc`
|  | time |
| --- | --- |
|real |   0m1.134s|
|user |   0m0.000s|
|sys  |   0m1.131s |

`CopyIoUring`
|  | time |
| --- | --- |
|real  |  0m3.792s |
|user  |  0m0.010s |
|sys   |  0m5.342s |

## Benchmarks

I have conducted some benchmarks to compare the performance of the different copying methods provided by this library. Here are the results:

### Test Environment

- CPU: AMD Ryzen 9 5900X 12-Core Processor 3.70 GHz
- RAM: 32GB DDR4
- Disk: Kingston NVMe 2TB
- OS: Ubuntu - WSL2
- Rust version: 1.75.0

### Test Methodology

We copied a 1GB file 10 times using each method and measured the time taken for each copy.

### Results

| Method | Time |
| --- | --- |
| `CopyLibc` | [1.0447s 1.0589s 1.0743s] |
| `CopyIoUring` | [1.3980s 1.4212s 1.4458s] |

As you can see, the `CopyLibc` method is faster than the `CopyIoUring` method in our tests, this is because io_uring is not good for secuencial I/O

Please note that these results may vary depending on the specific hardware and software configuration of your system.

### Running the Benchmarks

You can run the benchmarks yourself using the `cargo bench` command:

```bash
cargo bench
```