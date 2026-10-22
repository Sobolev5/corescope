fn main() {
    let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let target = out.ancestors().nth(3).unwrap();
    let source = "assets/webview2/WebView2Loader.dll";
    println!("cargo:rerun-if-changed={source}");
    std::fs::copy(source, target.join("WebView2Loader.dll")).expect("copy WebView2 loader");
    println!("cargo:rerun-if-changed=assets/hardware");
    std::fs::create_dir_all(target.join("hardware")).unwrap();
    for file in std::fs::read_dir("assets/hardware").unwrap() {
        let file = file.unwrap();
        if file.file_type().unwrap().is_file() {
            std::fs::copy(file.path(), target.join("hardware").join(file.file_name())).unwrap();
        }
    }
    println!("cargo:rerun-if-changed=assets/corescope.ico");
    let icon = std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap())
        .join("assets/corescope.ico");
    let rc = out.join("corescope.rc");
    let res = out.join("corescope.res");
    std::fs::write(
        &rc,
        format!(
            "1 ICON \"{}\"",
            icon.display().to_string().replace('\\', "/")
        ),
    )
    .unwrap();
    let mut c = if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("gnu") {
        let mut c = std::process::Command::new("windres");
        c.arg("-O")
            .arg("coff")
            .arg("-i")
            .arg(&rc)
            .arg("-o")
            .arg(&res);
        c
    } else {
        let mut c = std::process::Command::new("rc.exe");
        c.arg("/nologo")
            .arg(format!("/fo{}", res.display()))
            .arg(&rc);
        c
    };
    if c.status().is_ok_and(|s| s.success()) {
        println!("cargo:rustc-link-arg-bin=corescope={}", res.display());
    }
}
