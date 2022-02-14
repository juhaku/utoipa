use std::{
    env::{self, VarError},
    process::Command,
};

const SWAGGER_UI_DIST_ZIP: &str = "swagger-ui-4.5.0";

fn main() {
    println!("cargo:rerun-if-changed=res/{}.zip", SWAGGER_UI_DIST_ZIP);
    println!(
        "cargo:rustc-env=UTOIPA_SWAGGER_UI_VERSION={}",
        SWAGGER_UI_DIST_ZIP
    );

    let target_dir = env::var("CARGO_TARGET_DIR")
        .or_else(|_| env::var("CARGO_BUILD_TARGET_DIR"))
        .or_else(|_| -> Result<String, VarError> { Ok("target".to_string()) })
        .unwrap();
    println!("cargo:rustc-env=UTOIPA_SWAGGER_DIR={}", &target_dir);

    Command::new("unzip")
        .arg(&format!("res/{}.zip", SWAGGER_UI_DIST_ZIP))
        .arg(&format!("{}/dist/**", SWAGGER_UI_DIST_ZIP))
        .args(&["-d", &target_dir])
        .status()
        .unwrap();

    Command::new("sed")
        .args(&[
            "-i",
            r#"s|url: ".*",|{{urls}},|"#,
            &format!("target/{}/dist/index.html", SWAGGER_UI_DIST_ZIP),
        ])
        .status()
        .unwrap();
}
