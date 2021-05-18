use std::fs;

fn configure_memory_x(file: &str) {
    let filename = format!("memory/{}", file);

    println!("cargo:rerun-if-changed=memory.x");
    println!("cargo:rerun-if-changed={}", &filename);

    fs::copy(&filename, "memory.x").unwrap();
}

fn configure_runner(target: &str) {
    println!("cargo:rerun-if-changed={}", RUNNER_TARGET_FILE);

    const RUNNER_TARGET_FILE : &str = ".cargo/.runner-target";
    fs::write(RUNNER_TARGET_FILE, target).unwrap();
}

#[cfg(feature = "wgm160p")]
fn main() {
    configure_memory_x("wgm160p.x");
    configure_runner("wgm160p");
}

#[cfg(feature = "stm32f412_discovery")]
fn main() {
    println!("cargo:rerun-if-env-changed=LOADSTONE_USE_ALT_MEMORY");

    let use_alt_memory = match option_env!("LOADSTONE_USE_ALT_MEMORY") {
        None => false,
        Some("0") => false,
        Some("1") => true,
        _ => panic!("LOADSTONE_USE_ALT_MEMORY must be 0, 1 or undefined."),
    };

    let memory_file = if use_alt_memory {
        "stm32f412_discovery.alt.x"
    } else {
        "stm32f412_discovery.x"
    };

    configure_memory_x(memory_file);
    configure_runner("stm32f412_discovery");
}
