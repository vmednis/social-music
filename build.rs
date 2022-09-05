use std::process::Command;

fn main() {
    Command::new("./build-client.sh")
        .output()
        .expect("Failed to build client!");

    println!("cargo:rerun-if-changed=client/");
}
