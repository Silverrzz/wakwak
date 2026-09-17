use std::env;
use std::fs;

#[derive(Debug, Clone)]
pub struct SplitMix64 {
    pub state: u64,
}

impl SplitMix64 {
    #[inline]
    pub const fn new(state: u64) -> SplitMix64 {
        SplitMix64 { state }
    }

    #[inline]
    pub const fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e3779b97f4a7c15u64);

        let mut temp = self.state;
        temp = (temp ^ (temp >> 30)).wrapping_mul(0xbf58476d1ce4e5b9u64);
        temp = (temp ^ (temp >> 27)).wrapping_mul(0x94d049bb133111ebu64);

        temp ^ (temp >> 31)
    }
}

fn main() {
    //let net_src = env::var("EVALFILE").unwrap_or("nets/wakwak.nnue".to_string());
    let net_dest = env::var("OUT_DIR").unwrap() + "/wakwak.nnue";
    fs::write(&net_dest, [1; 26752]).unwrap();

    /*if !fs::exists(&net_src).unwrap() {
        panic!(
            "No network found! Use the Makefile or specify a path through the `EVALFILE` environment variable."
        );
    }

    fs::copy(&net_src, net_dest).unwrap();*/

    println!("cargo:rerun-if-env-changed=EVALFILE");
    //println!("cargo:rerun-if-changed={net_src}");
    println!("cargo:rerun-if-changed=build.rs");
}
