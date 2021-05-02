fn main() {
    println!("cargo:rerun-if-changed=HalideRuntime.h");
    println!("cargo:rerun-if-changed=libbrighter.a");

    let bindings = bindgen::Builder::default()
        .header("HalideRuntime.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .rustified_enum("halide.*_t")
        .allowlist_type("halide.*_t")
        .allowlist_function("halide.*")
        .generate_comments(false)
        .generate()
        .expect("Unable to generate bindings");

    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("runtime.rs"))
        .expect("Couldn't write bindings");

    let t = std::env::var("TEST");
    if t.is_ok() && t.unwrap() != "" {
        let halide_path = std::env::var("HALIDE_PATH")
            .unwrap_or_else(|_| format!("{}/halide", std::env::var("HOME").unwrap()));

        let build = halide_build::Build::new(halide_path, "./brighter")
            .generator(true)
            .source_file("brighter.cpp")
            .run_args(vec![
                "-g",
                "brighter",
                "-n",
                "libbrighter",
                "-o",
                ".",
                "-e",
                "static_library",
                "target=host",
            ]);

        build.build().unwrap();
        build.run().unwrap();

        println!("cargo:rustc-link-search=.");
        println!("cargo:rustc-link-lib=brighter");
    }
}
