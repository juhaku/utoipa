use std::process::Command;

const SWAGGER_UI_DIST_ZIP: &str = "swagger-ui-3.52.5";

fn main() {
    println!("cargo:rerun-if-changed={}.zip", SWAGGER_UI_DIST_ZIP);

    Command::new("unzip")
        .arg(&format!("{}.zip", SWAGGER_UI_DIST_ZIP))
        .arg(&format!("{}/dist/**", SWAGGER_UI_DIST_ZIP))
        .args(&["-d", "target"])
        .status()
        .unwrap();
}
