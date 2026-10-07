//! Build the owned OCCT bridge only when kernel is enabled; headless intent stays portable.
fn main() {
    #[cfg(feature = "kernel")]
    {
        let include = std::env::var("OCCT_INCLUDE_DIR")
            .expect("Set OCCT_INCLUDE_DIR (or build inside nix-shell)");
        let lib =
            std::env::var("OCCT_LIB_DIR").expect("Set OCCT_LIB_DIR (or build inside nix-shell)");
        cxx_build::bridge("src/kernel/bridge.rs")
            .file("src/kernel/native/occt.cpp")
            .include(".")
            .include(include)
            .std("c++17")
            .opt_level(1)
            .compile("confusion_occt");
        println!("cargo:rustc-link-search=native={lib}");
        for name in [
            "TKernel",
            "TKMath",
            "TKG2d",
            "TKG3d",
            "TKGeomBase",
            "TKBRep",
            "TKGeomAlgo",
            "TKTopAlgo",
            "TKPrim",
            "TKBO",
            "TKBool",
            "TKMesh",
        ] {
            println!("cargo:rustc-link-lib={name}");
        }
        for file in [
            "src/kernel/bridge.rs",
            "src/kernel/native/occt.cpp",
            "src/kernel/native/occt.hpp",
        ] {
            println!("cargo:rerun-if-changed={file}");
        }
        println!("cargo:rerun-if-env-changed=OCCT_INCLUDE_DIR");
        println!("cargo:rerun-if-env-changed=OCCT_LIB_DIR");
    }
}
