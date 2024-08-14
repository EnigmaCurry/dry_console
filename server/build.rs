use std::env;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

fn write_source(file: &mut std::fs::File, destination: &str, source: &str, file_type: &str) {
    writeln!(
        file,
        "        (\"{destination}\", include_bytes!(\"{source}\"), \"{file_type}\"),",
    )
    .unwrap();
}

fn write_font(file: &mut std::fs::File, dist_dir: &str, font: &str) {
    write_source(
        file,
        font.to_string().as_str(),
        format!("{dist_dir}{font}").as_str(),
        "font/woff2",
    );
}

fn main() {
    // Get the output directory from the environment variable
    let out_dir = env::var("OUT_DIR").unwrap();
    let project_root: String = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .parent()
        .unwrap()
        .as_os_str()
        .to_string_lossy()
        .to_string();
    let dest_path = Path::new(&out_dir).join("generated_includes.rs");
    let mut file = File::create(dest_path).unwrap();

    // Define the path to the directory containing assets
    let dist_dir = PathBuf::from(project_root.clone())
        .join("dist")
        .to_string_lossy()
        .to_string();
    let snippets_dir = PathBuf::from(&dist_dir)
        .join("snippets")
        .to_string_lossy()
        .to_string();

    // Write the start of the function definition to the generated file
    writeln!(
        file,
        "pub fn get_inline_files() -> Vec<(&'static str, &'static [u8], &'static str)> {{"
    )
    .unwrap();
    writeln!(file, "    vec![").unwrap();

    // Walk through the ../dist/snippets directory and find all inline0.js files
    for entry in WalkDir::new(snippets_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.path().is_file()
            && entry
                .file_name()
                .to_str()
                .map_or(false, |f| f.ends_with(".js"))
        {
            let asset_path = entry.path();

            write_source(
                &mut file,
                asset_path
                    .to_string_lossy()
                    .strip_prefix(&dist_dir.to_string())
                    .unwrap(),
                asset_path.to_string_lossy().to_string().as_str(),
                "application/javascript",
            );
        }
    }

    // App CSS
    write_source(
        &mut file,
        "/style.css",
        format!("{project_root}/frontend/style.css").as_str(),
        "text/css",
    );

    // Patternfly CSS
    // TODO: Tree shake this 1.5MB
    write_source(
        &mut file,
        "/patternfly.min.css",
        format!("{dist_dir}/patternfly.min.css").as_str(),
        "text/css",
    );

    write_source(
        &mut file,
        "/patternfly.min.css.map",
        format!("{dist_dir}/patternfly.min.css.map").as_str(),
        "application/octet-stream",
    );

    // // Patternfly fonts
    write_font(
        &mut file,
        &dist_dir,
        "/assets/fonts/webfonts/fa-solid-900.woff2",
    );
    write_font(&mut file, &dist_dir, "/assets/pficon/pf-v5-pficon.woff2");
    write_font(
        &mut file,
        &dist_dir,
        "/assets/fonts/RedHatText/RedHatText-Regular.woff2",
    );
    write_font(
        &mut file,
        &dist_dir,
        "/assets/fonts/RedHatText/RedHatText-Medium.woff2",
    );
    write_font(
        &mut file,
        &dist_dir,
        "/assets/fonts/RedHatMono/RedHatMono-Regular.woff2",
    );
    write_font(
        &mut file,
        &dist_dir,
        "/assets/fonts/RedHatMono/RedHatMono-Medium.woff2",
    );
    write_font(
        &mut file,
        &dist_dir,
        "/assets/fonts/RedHatDisplay/RedHatDisplay-Medium.woff2",
    );

    // Write the end of the function definition
    writeln!(file, "    ]").unwrap();
    writeln!(file, "}}").unwrap();
}
