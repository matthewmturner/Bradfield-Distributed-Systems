fn main() {
    prost_build::compile_protos(&["src/ipc/messages.proto"], &["src/"]).unwrap();
}
