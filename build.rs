use std::fs;

fn configure_memory_x(target: &str) {
    let filename = format!("memory/{}.x", target);
    fs::copy(&filename, "memory.x").unwrap();

    println!("cargo:rerun-if-changed=memory.x");
    println!("cargo:rerun-if-changed={}", filename);
}

fn configure_runner(target: &str) {
    const RUNNER_TARGET_FILE : &str = ".cargo/.runner-target";
    fs::write(RUNNER_TARGET_FILE, target).unwrap();

    println!("cargo:rerun-if-changed={}", RUNNER_TARGET_FILE);
}

macro_rules! configuration {
    ($target: expr) => {
        #[cfg(feature = $target)]
        fn main() {
            configure_memory_x($target);
            configure_runner($target);
        }
    };
}

configuration!("stm32f412_discovery");
configuration!("wgm160p");
