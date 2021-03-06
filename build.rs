use std::env;
use std::path::PathBuf;
use std::fs::read_dir;
use std::process::Command;
use std::fmt::Display;
use std::ffi::OsString;

#[allow(dead_code)]
fn err_to_panic<T, E: Display>(result: Result<T, E>) -> T {
    match result {
        Ok(x) => x,
        Err(e) => panic!("{}", e)
    }
}

fn run(command: &mut Command) {
    let string = format!("{:?}", command);
    let status = err_to_panic(command.status());
    if !status.success() {
        panic!("`{}` did not execute successfully", string);
    }
}

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let base_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let flite_dir = base_dir.join("clib/flite");
    let flite_copy_dir = out_dir.join("flite");
    let include_dir = flite_copy_dir.join("include");

    let opts = &fs_extra::dir::CopyOptions {
        overwrite: true,
        ..Default::default()
    };
    fs_extra::dir::copy(&flite_dir, &out_dir, &opts).unwrap();
    
    // Run configure, make, and make install
    env::set_current_dir(&flite_copy_dir).unwrap();
    run(Command::new("./configure")
        .arg("--disable-shared")
        .arg("--with-audio=none"));
    run(Command::new("make")
        .args(&["-C", "src"]));
    run(Command::new("make")
        .args(&["-C", "lang"]));

    // Generate bindings
    /*let mut bindings = bindgen::Builder::default();

    // Some nessesary headers aren't automatically added by make
    let extra_headers = [
        flite_dir.join("lang/cmulex/cmu_lex.h"),
        flite_dir.join("lang/cmu_us_kal/cmu_us_kal.c"),
        flite_dir.join("lang/usenglish/usenglish.h"),
    ];

    for header in &extra_headers {
        std::fs::copy(header, include_dir.join(header.file_name().unwrap())).unwrap();
    }

    // Add all headers to bindings to be safe
    for entry in read_dir(&include_dir).unwrap() {
        let entry = entry.unwrap().path();
        let extension = entry.extension();
        if extension == Some(&OsString::from("c")) || extension == Some(&OsString::from("h")) {
            bindings = bindings.header(entry.into_os_string().into_string().unwrap());
        }
    }

    bindings
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Couldn't write bindings!");
    */
    let lib_dir = read_dir(flite_copy_dir.join("build")).unwrap().next().unwrap().unwrap().path().join("lib");
    println!("cargo:rustc-link-search={}", lib_dir.as_os_str().to_str().unwrap());

    // Flite genertes multiple static libs, link to all of them
    for entry in read_dir(&lib_dir).unwrap() {
        println!("cargo:rustc-link-lib=static={}", entry.unwrap().path().file_stem().unwrap().to_str().unwrap().trim_start_matches("lib"));
    }
}