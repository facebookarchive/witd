cargo build
mkdir lib 2>/dev/null
rustc -g -C rpath -L target -L target/deps -o lib/libwit.a --cfg c_target --crate-type staticlib src/lib.rs
