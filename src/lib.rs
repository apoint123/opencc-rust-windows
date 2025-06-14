/*!
Open Chinese Convert(OpenCC, 開放中文轉換) binding for the Rust language for conversion between Traditional Chinese and Simplified Chinese.

## Compilation

To compile this crate, you need to compile the OpenCC C++ library first. You can install OpenCC in your operating system, or in somewhere in your file system. As for the latter, you need to set the following environment variables to link the OpenCC library:

* `OPENCC_LIB_DIRS`: The directories of library files, like `-L`. Use `:` to separate.
* `OPENCC_LIBS`: The library names that you want to link, like `-l`. Use `:` to separate. Typically, it contains **opencc:marisa**.
* `OPENCC_INCLUDE_DIRS`: The directories of header files, like `-i`. Use `:` to separate.
* `OPENCC_STATIC`: Whether to use `static` or `dylib`.
* `OPENCC_DYLIB_STDCPP`: If you use `static` linking, and your OpenCC library is compiled by the GNU C, this environment variable should be set.

## Examples

```rust
use opencc_rust_windows::{*, DefaultConfig};
use tempfile::tempdir;

let dictionary_dir = tempdir().unwrap();
let dictionary_path = dictionary_dir.path();

generate_static_dictionary(&dictionary_path, DefaultConfig::TW2SP).unwrap();

let config_path = dictionary_path.join(DefaultConfig::TW2SP.get_file_name());

let opencc = OpenCC::new(config_path).unwrap();

let s = opencc.convert("涼風有訊").unwrap();
assert_eq!("凉风有讯", &s);

let mut buffer = s;
opencc.convert_append("，秋月無邊", &mut buffer).unwrap();
assert_eq!("凉风有讯，秋月无边", &buffer);
```

```rust
use opencc_rust_windows::{*, DefaultConfig};
use tempfile::tempdir;

let dictionary_dir = tempdir().unwrap();
generate_static_dictionary(&dictionary_dir, DefaultConfig::S2TWP).unwrap();
let config_path = dictionary_dir.path().join(DefaultConfig::S2TWP.get_file_name());

let opencc = OpenCC::new(config_path).unwrap();

let s = opencc.convert("凉风有讯").unwrap();
assert_eq!("涼風有訊", &s);

let mut buffer = s;
opencc.convert_append("，秋月无边", &mut buffer).unwrap();
assert_eq!("涼風有訊，秋月無邊", &buffer);
```

## Static Dictionaries

Usually, OpenCC needs to be executed on an environment where OpenCC is installed. If you want to make it portable, you can enable the `static-dictionaries` feature.

```toml
[dependencies.opencc-rust-windows]
version = "*"
features = ["static-dictionaries"]
```
Then, the `generate_static_dictionary` and `generate_static_dictionaries` functions are available.

The default OpenCC dictionaries will be compiled into the binary file by `lazy_static_include` crate. And you can use the two functions to recover them on demand.

For example,

```rust
use opencc_rust_windows::*;
use std::path::Path;
use tempfile::tempdir;

let dir = tempdir().unwrap();
let output_path = dir.path();

generate_static_dictionary(&output_path, DefaultConfig::TW2SP).unwrap();

let config_path = output_path.join(DefaultConfig::TW2SP.get_file_name());
let opencc = OpenCC::new(config_path).unwrap();

assert_eq!("凉风有讯", &opencc.convert("涼風有訊").unwrap());
```
*/

#[cfg(feature = "static-dictionaries")]
use std::error::Error;
#[cfg(feature = "static-dictionaries")]
use std::fs::{self, File};
#[cfg(feature = "static-dictionaries")]
use std::io::Write;
use std::sync::Mutex;
use std::{
    ffi::{CStr, CString},
    path::Path,
};

use libc::{c_char, c_int, c_void, size_t};
use thiserror::Error;

unsafe extern "C" {
    pub fn opencc_open(config_file_path: *const c_char) -> *mut c_void;
    pub fn opencc_close(opencc: *mut c_void) -> c_int;
    pub fn opencc_convert_utf8(
        opencc: *mut c_void,
        input: *const c_char,
        length: size_t,
    ) -> *mut c_char;
    pub fn opencc_convert_utf8_to_buffer(
        opencc: *mut c_void,
        input: *const c_char,
        length: size_t,
        output: *mut c_char,
    ) -> size_t;
    pub fn opencc_convert_utf8_free(str: *mut c_char);
    pub fn opencc_error() -> *const c_char;
}

/// Default configs.
#[derive(Debug, Copy, Clone)]
pub enum DefaultConfig {
    /// Traditional Chinese (Hong Kong Standard) to Simplified Chinese
    HK2S,
    /// Traditional Chinese (Hong Kong Standard) to Traditional Chinese
    HK2T,
    /// New Japanese Kanji (Shinjitai) to Traditional Chinese Characters (Kyūjitai)
    JP2T,
    /// Simplified Chinese to Traditional Chinese
    S2T,
    /// Simplified Chinese to Traditional Chinese (Taiwan Standard)
    S2TW,
    /// Simplified Chinese to Traditional Chinese (Taiwan Standard) with Taiwanese idiom
    S2TWP,
    /// Traditional Chinese (OpenCC Standard) to Hong Kong Standard
    T2HK,
    /// Traditional Chinese Characters (Kyūjitai) to New Japanese Kanji (Shinjitai)
    T2JP,
    /// Traditional Chinese (OpenCC Standard) to Taiwan Standard
    T2TW,
    /// Traditional Chinese to Simplified Chinese
    T2S,
    /// Simplified Chinese to Traditional Chinese (Hong Kong Standard)
    S2HK,
    /// Traditional Chinese (Taiwan Standard) to Simplified Chinese
    TW2S,
    /// Traditional Chinese (Taiwan Standard) to Simplified Chinese with Mainland Chinese idiom
    TW2SP,
    /// Traditional Chinese (Taiwan Standard) to Traditional Chinese
    TW2T,
}

impl DefaultConfig {
    /// Get the file name for this default config.
    pub fn get_file_name(self) -> &'static str {
        match self {
            DefaultConfig::HK2S => "hk2s.json",
            DefaultConfig::HK2T => "hk2t.json",
            DefaultConfig::JP2T => "jp2t.json",
            DefaultConfig::S2HK => "s2hk.json",
            DefaultConfig::S2T => "s2t.json",
            DefaultConfig::S2TW => "s2tw.json",
            DefaultConfig::S2TWP => "s2twp.json",
            DefaultConfig::T2HK => "t2hk.json",
            DefaultConfig::T2JP => "t2jp.json",
            DefaultConfig::T2S => "t2s.json",
            DefaultConfig::T2TW => "t2tw.json",
            DefaultConfig::TW2S => "tw2s.json",
            DefaultConfig::TW2SP => "tw2sp.json",
            DefaultConfig::TW2T => "tw2t.json",
        }
    }
}

impl AsRef<Path> for DefaultConfig {
    fn as_ref(&self) -> &Path {
        Path::new(self.get_file_name())
    }
}

impl AsRef<str> for DefaultConfig {
    fn as_ref(&self) -> &str {
        self.get_file_name()
    }
}

/// Represents all errors that may occur
#[derive(Error, Debug)]
pub enum OpenCCError {
    /// Occurs when the supplied path cannot be converted to a C string, usually because it contains internal NULL characters.
    #[error("The configuration file path contains invalid characters")]
    InvalidConfigPath,

    /// Occurs when the OpenCC C library cannot be initialized with the given configuration file.
    /// This usually means that the configuration file path is wrong, or the file contents are malformed.
    /// The String contains the details from `opencc_error()`.
    #[error("Failed to create OpenCC instance: {0}")]
    NewInstanceFailed(String),

    /// Occurs when the input string to be converted contains an internal NULL character.
    #[error("The input string contains an invalid NULL character")]
    InputContainsNull,

    /// The OpenCC C library reported an error during string conversion.
    /// The String contains the details from `opencc_error()`.
    #[error("OpenCC conversion failed: {0}")]
    ConversionFailed(String),

    /// Occurs when the OpenCC C library returns an illegal UTF-8 byte sequence.
    #[error("OpenCC returned an invalid UTF-8 sequence")]
    InvalidUtf8,
}

/// OpenCC binding for Rust.
pub struct OpenCC {
    opencc: Mutex<*mut c_void>,
}

unsafe impl Send for OpenCC {}

unsafe impl Sync for OpenCC {}

impl OpenCC {
    /// Create a new OpenCC instance through a file provided by its path.
    pub fn new<P: AsRef<Path>>(config_file_path: P) -> Result<Self, OpenCCError> {
        let path_str = config_file_path
            .as_ref()
            .to_str()
            .ok_or(OpenCCError::InvalidConfigPath)?;

        let config_file_path_cstring =
            CString::new(path_str).map_err(|_| OpenCCError::InvalidConfigPath)?;

        // Call the C function directly to get a new pointer
        let opencc_ptr = unsafe { opencc_open(config_file_path_cstring.as_ptr()) };

        // Check if the returned pointer is valid
        if opencc_ptr.is_null() || opencc_ptr as isize == -1 {
            // If the build fails, get detailed error information from the C library
            let error_msg = unsafe {
                let err_ptr = opencc_error();
                if err_ptr.is_null() {
                    "Unknown error from OpenCC library".to_string()
                } else {
                    CStr::from_ptr(err_ptr).to_string_lossy().into_owned()
                }
            };
            return Err(OpenCCError::NewInstanceFailed(error_msg));
        }

        Ok(OpenCC {
            opencc: Mutex::new(opencc_ptr),
        })
    }

    /// Convert a string to another string.
    pub fn convert<S: AsRef<str>>(&self, input: S) -> Result<String, OpenCCError> {
        let input_ref = input.as_ref();

        let length = input_ref.len();
        let c_input = match CString::new(input_ref) {
            Ok(s) => s,
            Err(_) => return Err(OpenCCError::InputContainsNull),
        };

        // Get the lock and prepare to call the C function
        let handle = self.opencc.lock().unwrap();

        // Check if the handle has been initialized correctly
        if handle.is_null() {
            return Err(OpenCCError::NewInstanceFailed(
                "OpenCC instance is not valid or has been closed.".into(),
            ));
        }

        let result_ptr = unsafe { opencc_convert_utf8(*handle, c_input.as_ptr(), length) };

        if result_ptr.is_null() {
            // Get detailed error information provided by the OpenCC C library
            let error_msg = unsafe {
                let err_ptr = opencc_error();
                if err_ptr.is_null() {
                    "Unknown conversion error from OpenCC library".to_string()
                } else {
                    CStr::from_ptr(err_ptr).to_string_lossy().into_owned()
                }
            };
            return Err(OpenCCError::ConversionFailed(error_msg));
        }

        let result_cstr = unsafe { CStr::from_ptr(result_ptr) };
        let result = result_cstr.to_string_lossy().to_string();

        unsafe {
            opencc_convert_utf8_free(result_ptr);
        }

        Ok(result)
    }

    /// Convert a string to another string and store into a buffer.
    #[deprecated]
    pub fn convert_to_buffer<S: AsRef<str>>(&self, input: S, mut output: String) -> String {
        let input_ref = input.as_ref();
        let length = input_ref.len();
        let c_input = CString::new(input_ref).unwrap();

        let mut buffer: Vec<u8> = Vec::with_capacity(length * 3);
        let buffer_ptr = buffer.as_mut_ptr() as *mut c_char;

        let handle = self.opencc.lock().unwrap();

        let size =
            unsafe { opencc_convert_utf8_to_buffer(*handle, c_input.as_ptr(), length, buffer_ptr) };

        if size > 0 {
            unsafe {
                buffer.set_len(size);
            }

            let result_slice = std::str::from_utf8(&buffer).expect("OpenCC returned invalid UTF-8");
            output.push_str(result_slice);
        }

        output
    }

    /// Converts the input string and appends the result to the end of the `output` string.
    ///
    /// # Arguments
    ///
    /// * `input` - The string slice to be converted.
    /// * `output` - A mutable String to which the conversion result is appended.
    ///
    /// # Errors
    ///
    /// If the input contains NULL bytes, or if the C library conversion fails, an error is returned.
    pub fn convert_append<S: AsRef<str>>(
        &self,
        input: S,
        output: &mut String,
    ) -> Result<(), OpenCCError> {
        let input_ref = input.as_ref();
        let length = input_ref.len();

        // Check if the input contains a null byte
        let c_input = CString::new(input_ref).map_err(|_| OpenCCError::InputContainsNull)?;

        // Get the Lock
        let handle = self.opencc.lock().unwrap();
        if handle.is_null() {
            return Err(OpenCCError::NewInstanceFailed(
                "OpenCC instance is not valid.".into(),
            ));
        }

        // Create a temporary buffer to receive the output of the C function.
        // We preallocate space 3 times the input length, which is usually enough for UTF-8 conversion.
        let mut buffer: Vec<u8> = Vec::with_capacity(length * 3 + 1);
        let buffer_ptr = buffer.as_mut_ptr() as *mut c_char;

        // Call the C function
        let size =
            unsafe { opencc_convert_utf8_to_buffer(*handle, c_input.as_ptr(), length, buffer_ptr) };

        // Check if the conversion was successful
        if size == (size_t::MAX) {
            // size_t::MAX usually means -1 or error in C libraries
            let error_msg = unsafe {
                let err_ptr = opencc_error();
                if err_ptr.is_null() {
                    "Unknown conversion error".to_string()
                } else {
                    CStr::from_ptr(err_ptr).to_string_lossy().into_owned()
                }
            };
            return Err(OpenCCError::ConversionFailed(error_msg));
        }

        if size > 0 {
            unsafe {
                buffer.set_len(size);
            }

            let result_slice =
                std::str::from_utf8(&buffer).map_err(|_| OpenCCError::InvalidUtf8)?;

            output.push_str(result_slice);
        }

        Ok(())
    }
}

impl Drop for OpenCC {
    fn drop(&mut self) {
        let handle = self.opencc.get_mut().unwrap();
        if !handle.is_null() {
            unsafe {
                opencc_close(*handle);
            }
        }
    }
}

#[cfg(feature = "static-dictionaries")]
use phf::{Map, phf_map};

#[cfg(feature = "static-dictionaries")]
struct StaticDictionary(&'static str, &'static [u8]);

#[cfg(feature = "static-dictionaries")]
static DICTIONARIES: StaticDictionaryData = StaticDictionaryData {
    hk2s_json: StaticDictionary(
        "hk2s.json",
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/opencc/hk2s.json")),
    ),
    hk2t_json: StaticDictionary(
        "hk2t.json",
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/opencc/hk2t.json")),
    ),
    hk_variants_ocd: StaticDictionary(
        "HKVariants.ocd2",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/opencc/HKVariants.ocd2"
        )),
    ),
    hk_variants_rev_ocd: StaticDictionary(
        "HKVariantsRev.ocd2",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/opencc/HKVariantsRev.ocd2"
        )),
    ),
    hk_variants_rev_phrases_ocd: StaticDictionary(
        "HKVariantsRevPhrases.ocd2",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/opencc/HKVariantsRevPhrases.ocd2"
        )),
    ),
    jp2t_json: StaticDictionary(
        "jp2t.json",
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/opencc/jp2t.json")),
    ),
    jp_shinjitai_characters_ocd: StaticDictionary(
        "JPShinjitaiCharacters.ocd2",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/opencc/JPShinjitaiCharacters.ocd2"
        )),
    ),
    jp_shinjitai_phrases_ocd: StaticDictionary(
        "JPShinjitaiPhrases.ocd2",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/opencc/JPShinjitaiPhrases.ocd2"
        )),
    ),
    jp_variants_ocd: StaticDictionary(
        "JPVariants.ocd2",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/opencc/JPVariants.ocd2"
        )),
    ),
    jp_variants_rev_ocd: StaticDictionary(
        "JPVariantsRev.ocd2",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/opencc/JPVariantsRev.ocd2"
        )),
    ),
    s2hk_json: StaticDictionary(
        "s2hk.json",
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/opencc/s2hk.json")),
    ),
    s2t_json: StaticDictionary(
        "s2t.json",
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/opencc/s2t.json")),
    ),
    s2tw_json: StaticDictionary(
        "s2tw.json",
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/opencc/s2tw.json")),
    ),
    s2twp_json: StaticDictionary(
        "s2twp.json",
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/opencc/s2twp.json")),
    ),
    st_characters_ocd: StaticDictionary(
        "STCharacters.ocd2",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/opencc/STCharacters.ocd2"
        )),
    ),
    st_phrases_ocd: StaticDictionary(
        "STPhrases.ocd2",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/opencc/STPhrases.ocd2"
        )),
    ),
    t2hk_json: StaticDictionary(
        "t2hk.json",
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/opencc/t2hk.json")),
    ),
    t2jp_json: StaticDictionary(
        "t2jp.json",
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/opencc/t2jp.json")),
    ),
    t2s_json: StaticDictionary(
        "t2s.json",
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/opencc/t2s.json")),
    ),
    t2tw_json: StaticDictionary(
        "t2tw.json",
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/opencc/t2tw.json")),
    ),
    ts_characters_ocd: StaticDictionary(
        "TSCharacters.ocd2",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/opencc/TSCharacters.ocd2"
        )),
    ),
    ts_phrases_ocd: StaticDictionary(
        "TSPhrases.ocd2",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/opencc/TSPhrases.ocd2"
        )),
    ),
    tw2s_json: StaticDictionary(
        "tw2s.json",
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/opencc/tw2s.json")),
    ),
    tw2sp_json: StaticDictionary(
        "tw2sp.json",
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/opencc/tw2sp.json")),
    ),
    tw2t_json: StaticDictionary(
        "tw2t.json",
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/opencc/tw2t.json")),
    ),
    tw_phrases_ocd: StaticDictionary(
        "TWPhrases.ocd2",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/opencc/TWPhrases.ocd2"
        )),
    ),
    tw_phrases_rev_ocd: StaticDictionary(
        "TWPhrasesRev.ocd2",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/opencc/TWPhrasesRev.ocd2"
        )),
    ),
    tw_variants_ocd: StaticDictionary(
        "TWVariants.ocd2",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/opencc/TWVariants.ocd2"
        )),
    ),
    tw_variants_rev_ocd: StaticDictionary(
        "TWVariantsRev.ocd2",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/opencc/TWVariantsRev.ocd2"
        )),
    ),
    tw_variants_rev_phrases_ocd: StaticDictionary(
        "TWVariantsRevPhrases.ocd2",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/opencc/TWVariantsRevPhrases.ocd2"
        )),
    ),
};

#[cfg(feature = "static-dictionaries")]
struct StaticDictionaryData {
    hk2s_json: StaticDictionary,
    hk2t_json: StaticDictionary,
    hk_variants_ocd: StaticDictionary,
    hk_variants_rev_ocd: StaticDictionary,
    hk_variants_rev_phrases_ocd: StaticDictionary,
    jp2t_json: StaticDictionary,
    jp_shinjitai_characters_ocd: StaticDictionary,
    jp_shinjitai_phrases_ocd: StaticDictionary,
    jp_variants_ocd: StaticDictionary,
    jp_variants_rev_ocd: StaticDictionary,
    s2hk_json: StaticDictionary,
    s2t_json: StaticDictionary,
    s2tw_json: StaticDictionary,
    s2twp_json: StaticDictionary,
    st_characters_ocd: StaticDictionary,
    st_phrases_ocd: StaticDictionary,
    t2hk_json: StaticDictionary,
    t2jp_json: StaticDictionary,
    t2s_json: StaticDictionary,
    t2tw_json: StaticDictionary,
    ts_characters_ocd: StaticDictionary,
    ts_phrases_ocd: StaticDictionary,
    tw2s_json: StaticDictionary,
    tw2sp_json: StaticDictionary,
    tw2t_json: StaticDictionary,
    tw_phrases_ocd: StaticDictionary,
    tw_phrases_rev_ocd: StaticDictionary,
    tw_variants_ocd: StaticDictionary,
    tw_variants_rev_ocd: StaticDictionary,
    tw_variants_rev_phrases_ocd: StaticDictionary,
}

#[cfg(feature = "static-dictionaries")]
static CONFIG_MAP: Map<&'static str, &'static [&'static StaticDictionary]> = phf_map! {
    "hk2s.json" => &[&DICTIONARIES.hk2s_json, &DICTIONARIES.ts_phrases_ocd, &DICTIONARIES.hk_variants_rev_phrases_ocd, &DICTIONARIES.hk_variants_rev_ocd, &DICTIONARIES.ts_characters_ocd],
    "hk2t.json" => &[&DICTIONARIES.hk2t_json, &DICTIONARIES.hk_variants_rev_phrases_ocd, &DICTIONARIES.hk_variants_rev_ocd],
    "jp2t.json" => &[&DICTIONARIES.jp2t_json, &DICTIONARIES.jp_shinjitai_phrases_ocd, &DICTIONARIES.jp_shinjitai_characters_ocd, &DICTIONARIES.jp_variants_rev_ocd],
    "s2hk.json" => &[&DICTIONARIES.s2hk_json, &DICTIONARIES.st_phrases_ocd, &DICTIONARIES.st_characters_ocd, &DICTIONARIES.hk_variants_ocd],
    "s2t.json" => &[&DICTIONARIES.s2t_json, &DICTIONARIES.st_phrases_ocd, &DICTIONARIES.st_characters_ocd],
    "s2tw.json" => &[&DICTIONARIES.s2tw_json, &DICTIONARIES.st_phrases_ocd, &DICTIONARIES.st_characters_ocd, &DICTIONARIES.tw_variants_ocd],
    "s2twp.json" => &[&DICTIONARIES.s2twp_json, &DICTIONARIES.st_phrases_ocd, &DICTIONARIES.st_characters_ocd, &DICTIONARIES.tw_phrases_ocd, &DICTIONARIES.tw_variants_ocd],
    "t2hk.json" => &[&DICTIONARIES.t2hk_json, &DICTIONARIES.hk_variants_ocd],
    "t2jp.json" => &[&DICTIONARIES.t2jp_json, &DICTIONARIES.jp_variants_ocd],
    "t2s.json" => &[&DICTIONARIES.t2s_json, &DICTIONARIES.ts_phrases_ocd, &DICTIONARIES.ts_characters_ocd],
    "t2tw.json" => &[&DICTIONARIES.t2tw_json, &DICTIONARIES.tw_variants_ocd],
    "tw2s.json" => &[&DICTIONARIES.tw2s_json, &DICTIONARIES.ts_phrases_ocd, &DICTIONARIES.tw_variants_rev_phrases_ocd, &DICTIONARIES.tw_variants_rev_ocd, &DICTIONARIES.ts_characters_ocd],
    "tw2sp.json" => &[&DICTIONARIES.tw2sp_json, &DICTIONARIES.ts_phrases_ocd, &DICTIONARIES.tw_phrases_rev_ocd, &DICTIONARIES.tw_variants_rev_phrases_ocd, &DICTIONARIES.tw_variants_rev_ocd, &DICTIONARIES.ts_characters_ocd],
    "tw2t.json" => &[&DICTIONARIES.tw2t_json, &DICTIONARIES.tw_variants_rev_phrases_ocd, &DICTIONARIES.tw_variants_rev_ocd],
};

#[cfg(feature = "static-dictionaries")]
fn generate_static_dictionary_inner<P: AsRef<Path>>(
    path: P,
    config: DefaultConfig,
) -> Result<(), Box<dyn Error>> {
    let path = path.as_ref();
    let config_filename = config.get_file_name();

    if let Some(dictionaries_to_write) = CONFIG_MAP.get(config_filename) {
        for data in *dictionaries_to_write {
            let output_path = path.join(data.0);

            if !output_path.exists() {
                let mut file = File::create(output_path)?;
                file.write_all(data.1)?;
                file.flush()?;
            }
        }
    } else {
        return Err(format!("Unsupported or unknown default config: {}", config_filename).into());
    }

    Ok(())
}

#[cfg(feature = "static-dictionaries")]
/// Generate files for a specific dictionary. These files are used for opening a new OpenCC instance.
pub fn generate_static_dictionary<P: AsRef<Path>>(
    path: P,
    config: DefaultConfig,
) -> Result<(), Box<dyn Error>> {
    let path = path.as_ref();

    if path.exists() {
        if !path.is_dir() {
            return Err(format!(
                "The path '{}' exists but is not a directory.",
                path.display()
            )
            .into());
        }
    } else {
        fs::create_dir_all(path)?;
    }

    generate_static_dictionary_inner(path, config)
}

#[cfg(feature = "static-dictionaries")]
/// Generate files for specific dictionaries. These files are used for opening a new OpenCC instance.
pub fn generate_static_dictionaries<P: AsRef<Path>>(
    path: P,
    configs: &[DefaultConfig],
) -> Result<(), Box<dyn Error>> {
    let path = path.as_ref();

    if path.exists() {
        if !path.is_dir() {
            return Err("The path of static dictionaries needs to be a directory.".into());
        }
    } else {
        match fs::create_dir_all(path) {
            Ok(_) => (),
            Err(_) => return Err("Cannot create new directories.".into()),
        }
    }

    for config in configs.iter().copied() {
        generate_static_dictionary_inner(path, config)?
    }

    Ok(())
}
