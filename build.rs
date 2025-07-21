fn main() {
    println!(
        "cargo:rustc-env=HOMELABD_VERSION={}",
        env!("CARGO_PKG_VERSION")
    );

    prost_build::compile_protos(&["proto/homelabd.proto"], &["proto"]).unwrap();
}
