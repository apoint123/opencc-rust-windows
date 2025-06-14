# OpenCC Rust for Windows

This is a fork of the original [opencc-rust](https://github.com/magiclen/opencc-rust), specially optimized to provide a better experience on the **Windows (x86_64-pc-windows-msvc)** platform.

## Core Features

  * **Out-of-the-Box on Windows**: On the `x86_64-pc-windows-msvc` target, the build script automatically links against the vendored OpenCC libraries included in this project. No manual setup is required.
  * **Static Linking by Default**: On Windows, static linking is used by default, producing an executable that does not depend on external OpenCC DLLs.
  * **Multi-Platform Compatibility**: While prioritizing Windows, this crate maintains the original methods for compiling on other platforms like Linux and macOS via `pkg-config` or `vcpkg`.

## Compilation

### Windows (x86_64-pc-windows-msvc)

**No additional configuration is needed.**

The build script will automatically detect your target platform and link the pre-compiled OpenCC libraries bundled with the project. Simply run `cargo build`.

### Other Platforms (Linux, macOS, etc.)

For non-Windows MSVC platforms, you first need to install the OpenCC C++ library on your system. You can do this via a system package manager (e.g., `apt`, `yum`, `brew`) or by compiling from the source.

The following methods are supported for linking the OpenCC library:

1.  **vcpkg (Recommended)**: If you are using MSVC for a target other than `x86_64-pc-windows-msvc`, or if you use vcpkg on other platforms, the build script will automatically try to find OpenCC via vcpkg.
2.  **pkg-config (Default for Linux/macOS)**: The build script will use `pkg-config` to find OpenCC automatically.
3.  **Environment Variables**: If the above methods are not applicable, you can manually specify the library locations by setting the following environment variables:
      * `OPENCC_LIB_DIRS`: The directories of library files (`-L`).
      * `OPENCC_LIBS`: The names of the libraries to link (`-l`), typically `opencc:marisa:darts`.
      * `OPENCC_INCLUDE_DIRS`: The directories of header files (`-i`).
      * `OPENCC_STATIC`: Set to `1` or `true` to force static linking.

## Usage Examples

The following examples demonstrate how to use the `static-dictionaries` feature to perform conversions. This is the most convenient approach, as it compiles all the necessary dictionary files directly into your program.

First, add the dependencies to your `Cargo.toml`:

```toml
[dependencies]
opencc-rust-windows = "1.2.0"
tempfile = "3" # Used for creating a temporary directory in the example
```

### Traditional to Simplified (TW2SP)

```rust
use opencc_rust_windows::{*, DefaultConfig};
use tempfile::tempdir;
use std::path::Path;

// 1. Create a temporary directory to store the generated dictionary files.
let dictionary_dir = tempdir().unwrap();
let dictionary_path = dictionary_dir.path();

// 2. Generate the dictionary and configuration files required for `tw2sp.json`.
generate_static_dictionary(&dictionary_path, DefaultConfig::TW2SP).unwrap();

// 3. Construct the full path to the configuration file.
let config_path = dictionary_path.join(DefaultConfig::TW2SP.get_file_name());

// 4. Create an OpenCC instance using the config file.
let opencc = OpenCC::new(config_path).unwrap();

// 5. Perform the conversion.
let text_to_convert = "涼風有訊，秋月無邊";
let converted_text = opencc.convert(text_to_convert).unwrap();

assert_eq!("凉风有讯，秋月无边", &converted_text);

println!("`{}` => `{}`", text_to_convert, converted_text);
```

### Simplified to Traditional (S2TWP)

```rust
use opencc_rust_windows::{*, DefaultConfig};
use tempfile::tempdir;
use std::path::Path;

let dictionary_dir = tempdir().unwrap();
let dictionary_path = dictionary_dir.path();

generate_static_dictionary(&dictionary_path, DefaultConfig::S2TWP).unwrap();

let config_path = dictionary_path.join(DefaultConfig::S2TWP.get_file_name());

let opencc = OpenCC::new(config_path).unwrap();

let text_to_convert = "凉风有讯，秋月无边";

let mut buffer = opencc.convert("凉风有讯").unwrap();
opencc.convert_append("，秋月无边", &mut buffer).unwrap();

assert_eq!("涼風有訊，秋月無邊", &buffer);

println!("`{}` => `{}`", text_to_convert, buffer);
```

## Crates.io

[https://crates.io/crates/opencc-rust-windows](https://crates.io/crates/opencc-rust-windows)

## Documentation

[https://docs.rs/opencc-rust-windows](https://docs.rs/opencc-rust-windows)

## License

[Apache-2.0](LICENSE)