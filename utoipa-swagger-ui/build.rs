use std::{
    env,
    error::Error,
    fs::{self, File},
    io,
    path::PathBuf,
};

use regex::Regex;
use zip::{result::ZipError, ZipArchive};

/// the following env variables control the build process:
/// 1. SWAGGER_UI_DOWNLOAD_URL:
/// + the url from where to download the swagger-ui zip file
/// + default value is SWAGGER_UI_DOWNLOAD_URL_DEFAULT
/// + for other versions, check https://github.com/swagger-api/swagger-ui/tags
/// 2. SWAGGER_UI_OVERWRITE_FOLDER
/// + absolute path to a folder containing files to overwrite the default swagger-ui files

const SWAGGER_UI_DOWNLOAD_URL_DEFAULT: &str =
    "https://github.com/swagger-api/swagger-ui/archive/refs/tags/v5.17.3.zip";

fn main() {
    let target_dir = env::var("OUT_DIR").unwrap();
    println!("OUT_DIR: {}", target_dir);

    let url =
        env::var("SWAGGER_UI_DOWNLOAD_URL").unwrap_or(SWAGGER_UI_DOWNLOAD_URL_DEFAULT.to_string());

    println!("SWAGGER_UI_DOWNLOAD_URL: {}", url);
    let zip_filename = url.split('/').last().unwrap().to_string();
    let zip_path = [&target_dir, &zip_filename].iter().collect::<PathBuf>();

    if !zip_path.exists() {
        println!("start download to : {:?}", zip_path);
        download_file(&url, zip_path.clone()).unwrap();
    } else {
        println!("already downloaded: {:?}", zip_path);
    }

    println!("cargo:rerun-if-changed={:?}", zip_path.clone());

    let swagger_ui_zip =
        File::open([&target_dir, &zip_filename].iter().collect::<PathBuf>()).unwrap();

    let mut zip = ZipArchive::new(swagger_ui_zip).unwrap();

    let zip_top_level_folder = extract_within_path(&mut zip, &target_dir).unwrap();
    println!("zip_top_level_folder: {:?}", zip_top_level_folder);

    replace_default_url_with_config(&target_dir, &zip_top_level_folder);

    write_embed_code(&target_dir, &zip_top_level_folder);

    let overwrite_folder =
        PathBuf::from(env::var("SWAGGER_UI_OVERWRITE_FOLDER").unwrap_or("overwrite".to_string()));

    if overwrite_folder.exists() {
        println!("SWAGGER_UI_OVERWRITE_FOLDER: {:?}", overwrite_folder);

        for entry in fs::read_dir(overwrite_folder).unwrap() {
            let entry = entry.unwrap();
            let path_in = entry.path();
            println!("replacing file: {:?}", path_in.clone());
            overwrite_target_file(&target_dir, &zip_top_level_folder, path_in);
        }
    } else {
        println!(
            "SWAGGER_UI_OVERWRITE_FOLDER not found: {:?}",
            overwrite_folder
        );
    }
}

fn extract_within_path(zip: &mut ZipArchive<File>, target_dir: &str) -> Result<String, ZipError> {
    let mut zip_top_level_folder = String::new();

    for index in 0..zip.len() {
        let mut file = zip.by_index(index)?;
        let filepath = file
            .enclosed_name()
            .ok_or(ZipError::InvalidArchive("invalid path file"))?;

        if index == 0 {
            zip_top_level_folder = filepath
                .iter()
                .take(1)
                .map(|x| x.to_str().unwrap_or_default())
                .collect::<String>();
        }

        let folder = filepath
            .iter()
            .skip(1)
            .take(1)
            .map(|x| x.to_str().unwrap_or_default())
            .collect::<String>();

        if folder == "dist" {
            let directory = [&target_dir].iter().collect::<PathBuf>();
            let out_path = directory.join(filepath);

            if file.name().ends_with('/') {
                fs::create_dir_all(&out_path)?;
            } else {
                if let Some(p) = out_path.parent() {
                    if !p.exists() {
                        fs::create_dir_all(p)?;
                    }
                }
                let mut out_file = fs::File::create(&out_path)?;
                io::copy(&mut file, &mut out_file)?;
            }
            // Get and Set permissions
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = file.unix_mode() {
                    fs::set_permissions(&out_path, fs::Permissions::from_mode(mode))?;
                }
            }
        }
    }

    Ok(zip_top_level_folder)
}

fn replace_default_url_with_config(target_dir: &str, zip_top_level_folder: &str) {
    let regex = Regex::new(r#"(?ms)url:.*deep.*true,"#).unwrap();

    let path = [
        target_dir,
        zip_top_level_folder,
        "dist",
        "swagger-initializer.js",
    ]
    .iter()
    .collect::<PathBuf>();

    let mut swagger_initializer = fs::read_to_string(&path).unwrap();
    swagger_initializer = swagger_initializer.replace("layout: \"StandaloneLayout\"", "");

    let replaced_swagger_initializer = regex.replace(&swagger_initializer, "{{config}},");

    fs::write(&path, replaced_swagger_initializer.as_ref()).unwrap();
}

fn write_embed_code(target_dir: &str, zip_top_level_folder: &str) {
    let contents = format!(
        r#"
// This file is auto-generated during compilation, do not modify
#[derive(RustEmbed)]
#[folder = r"{}/{}/dist/"]
struct SwaggerUiDist;
"#,
        target_dir, zip_top_level_folder
    );
    let path = [target_dir, "embed.rs"].iter().collect::<PathBuf>();
    fs::write(path, contents).unwrap();
}

fn download_file(url: &str, path: PathBuf) -> Result<(), Box<dyn Error>> {
    let mut response = reqwest::blocking::get(url)?;
    let mut file = File::create(path)?;
    io::copy(&mut response, &mut file)?;
    Ok(())
}

fn overwrite_target_file(target_dir: &str, swagger_ui_dist_zip: &str, path_in: PathBuf) {
    let filename = path_in.file_name().unwrap().to_str().unwrap();
    println!("overwrite file: {:?}", path_in.file_name().unwrap());

    let content = fs::read_to_string(path_in.clone());

    match content {
        Ok(content) => {
            let path = [target_dir, swagger_ui_dist_zip, "dist", filename]
                .iter()
                .collect::<PathBuf>();

            fs::write(path, content).unwrap();
        }
        Err(_) => {
            println!("cannot read content from file: {:?}", path_in);
        }
    }
}
