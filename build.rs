use std::env;
use std::fs;

fn main() {
    let net_src = env::var("EVALFILE").unwrap_or("nets/wakwak.nnue".to_string());
    let net_dest = env::var("OUT_DIR").unwrap() + "/wakwak.nnue";

    if !fs::exists(&net_src).unwrap() {
        panic!(
            "No network found! Use the Makefile or specify a path through the `EVALFILE` environment variable."
        );
    }

    fs::copy(&net_src, net_dest).unwrap();

    println!("cargo:rerun-if-env-changed=EVALFILE");
    println!("cargo:rerun-if-changed={net_src}");
    println!("cargo:rerun-if-changed=build.rs");
}
