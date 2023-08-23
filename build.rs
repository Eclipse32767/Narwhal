use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=src/*");
    Command::new("xtr").arg("-o").arg("resources/messages.po").arg("src/main.rs").output().unwrap();
}