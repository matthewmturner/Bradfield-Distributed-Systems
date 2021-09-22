use prost_build;

fn main() {
    // let mut prost_build = prost_build::Config::new();
    // prost_build.type_attribute("blue.ipc.message.Store", "#[derive(Copy)]");
    // prost_build
    //     .compile_protos(&["src/store.proto"], &["src/"])
    //     .unwrap()

    prost_build::compile_protos(&["src/ipc/messages.proto"], &["src/"]).unwrap();
}
