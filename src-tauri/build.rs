fn main() {
    for variable in ["BUILD_COMMIT", "BUILD_DATE", "BUILD_TAG"] {
        println!("cargo:rerun-if-env-changed={variable}");
    }
    tauri_build::build()
}
