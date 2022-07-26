use std::{
    cmp::Ordering,
    env::{self, VarError},
    fs::{self, File},
    io,
    path::PathBuf,
};

use lazy_static::lazy_static;
use regex::Regex;
use zip::{result::ZipError, ZipArchive};

const SWAGGER_UI_DIST_ZIP: &str = "swagger-ui-4.10.3";

fn main() {
    println!("cargo:rerun-if-changed=res/{}.zip", SWAGGER_UI_DIST_ZIP);
    println!(
        "cargo:rustc-env=UTOIPA_SWAGGER_UI_VERSION={}",
        SWAGGER_UI_DIST_ZIP
    );

    let target_dir = env::var("CARGO_TARGET_DIR")
        .or_else(|_| env::var("CARGO_BUILD_TARGET_DIR"))
        .or_else(|_| -> Result<String, VarError> {
            let manifest_dir = env::var("CARGO_MANIFEST_DIR")?;
            if manifest_dir.contains("/package") {
                Ok("target".to_string())
            } else {
                Ok("../target".to_string())
            }
        })
        .unwrap();
    println!("cargo:rustc-env=UTOIPA_SWAGGER_DIR={}", &target_dir);

    let swagger_ui_zip = File::open(
        ["res", &format!("{}.zip", SWAGGER_UI_DIST_ZIP)]
            .iter()
            .collect::<PathBuf>(),
    )
    .unwrap();

    let mut zip = ZipArchive::new(swagger_ui_zip).unwrap();
    extract_within_path(&mut zip, [SWAGGER_UI_DIST_ZIP, "dist"], &target_dir).unwrap();

    replace_default_url_with_config(&target_dir);
}

fn extract_within_path<const N: usize>(
    zip: &mut ZipArchive<File>,
    path_segments: [&str; N],
    target_dir: &str,
) -> Result<(), ZipError> {
    for index in 0..zip.len() {
        let mut file = zip.by_index(index)?;
        let filepath = file
            .enclosed_name()
            .ok_or(ZipError::InvalidArchive("invalid path file"))?;

        if filepath
            .iter()
            .take(2)
            .map(|s| s.to_str().unwrap_or_default())
            .cmp(path_segments)
            == Ordering::Equal
        {
            let directory = [&target_dir].iter().collect::<PathBuf>();
            let outpath = directory.join(filepath);

            if file.name().ends_with('/') {
                fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(&p)?;
                    }
                }
                let mut outfile = fs::File::create(&outpath)?;
                io::copy(&mut file, &mut outfile)?;
            }
            // Get and Set permissions
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = file.unix_mode() {
                    fs::set_permissions(&outpath, fs::Permissions::from_mode(mode))?;
                }
            }
        }
    }

    Ok(())
}

fn replace_default_url_with_config(target_dir: &str) {
    lazy_static! {
        static ref RE: Regex = Regex::new(r#"(?ms)url:.*deep.*true,"#).unwrap();
    }

    let path = [
        target_dir,
        SWAGGER_UI_DIST_ZIP,
        "dist",
        "swagger-initializer.js",
    ]
    .iter()
    .collect::<PathBuf>();

    let mut swagger_initializer = fs::read_to_string(&path).unwrap();
    swagger_initializer = swagger_initializer.replace("layout: \"StandaloneLayout\"", "");

    let replaced_swagger_initializer = RE.replace(&swagger_initializer, "{{config}},");

    fs::write(&path, replaced_swagger_initializer.as_ref()).unwrap();
}
