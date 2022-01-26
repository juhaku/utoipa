use std::process::Command;

const SWAGGER_UI_DIST_ZIP: &str = "swagger-ui-4.3.0";

fn main() {
    println!("cargo:rerun-if-changed=res/{}.zip", SWAGGER_UI_DIST_ZIP);

    Command::new("unzip")
        .arg(&format!("res/{}.zip", SWAGGER_UI_DIST_ZIP))
        .arg(&format!("{}/dist/**", SWAGGER_UI_DIST_ZIP))
        .args(&["-d", "target"])
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
