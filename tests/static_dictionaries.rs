#![cfg(feature = "static-dictionaries")]

use opencc_rust_windows::{DefaultConfig, OpenCC};
use std::fs;
use tempfile::tempdir;

#[test]
fn generate_static_dictionary() {
    let dir = tempdir().expect("Failed to create temp dir");
    let output_path = dir.path();

    match opencc_rust_windows::generate_static_dictionary(output_path, DefaultConfig::TW2SP) {
        Ok(_) => println!("The dictionary file was created successfully."),
        Err(e) => panic!("Failed to generate dictionary file: {}", e),
    }

    let entries = fs::read_dir(output_path)
        .unwrap()
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    println!("Archives in the directory: {:?}", entries);

    let s = String::from("無");

    let config_filename = DefaultConfig::TW2SP.get_file_name();
    let config_path = output_path.join(config_filename);

    println!(
        "Prepare to initialize OpenCC using configuration file '{}'...",
        config_path.display()
    );

    let opencc = OpenCC::new(&config_path).expect("OpenCC initialization failed");

    println!("OpenCC initialization successful!");

    assert_eq!("无", &opencc.convert(s).unwrap());

    println!("Conversion Successful!");
}
