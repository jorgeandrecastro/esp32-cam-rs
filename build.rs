fn main() {
    // Propagates the ESP-IDF build environment (linker args, cfgs) to this crate.
    embuild::espidf::sysenv::output();
}