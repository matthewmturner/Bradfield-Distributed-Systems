use prost_build;

fn main() {
    prost_build::compile_protos(&["src/store.proto"], &["src/"]).unwrap()
}
