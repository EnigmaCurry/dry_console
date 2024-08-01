use std::env;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

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
    let dist_path = Path::new(&dist_dir);
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

            writeln!(
                file,
                "        (\"{}\", include_bytes!(\"{}\"), \"application/javascript\"),",
                asset_path
                    .to_string_lossy()
                    .strip_prefix(&format!("{dist_dir}"))
                    .unwrap(),
                asset_path.to_string_lossy()
            )
            .unwrap();
        }
    }

    // App CSS
    writeln!(
        file, "{}",
        format!("        (\"/style.css\", include_bytes!(\"{project_root}/frontend/style.css\"), \"text/css\"),"),
    )
    .unwrap();

    // Patternfly CSS
    // TODO: Tree shake this 1.5MB
    writeln!(
        file, "{}",
        format!("        (\"/patternfly.min.css\", include_bytes!(\"{dist_dir}/patternfly.min.css\"), \"text/css\"),"),
    )
    .unwrap();
    writeln!(
        file, "{}",
        format!("        (\"/patternfly.min.css.map\", include_bytes!(\"{dist_dir}/patternfly.min.css.map\"), \"application/octet-stream\"),")
    )
    .unwrap();

    // Patternfly fonts
    writeln!(
        file, "{}",
        format!("        (\"/assets/fonts/webfonts/fa-solid-900.woff2\", include_bytes!(\"{dist_dir}/assets/fonts/webfonts/fa-solid-900.woff2\"), \"font/woff2\"),"),
    )
    .unwrap();
    writeln!(
        file, "{}",
        format!("        (\"/assets/fonts/RedHatText/RedHatText-Regular.woff2\", include_bytes!(\"{dist_dir}/assets/fonts/RedHatText/RedHatText-Regular.woff2\"), \"font/woff2\"),"),
    )
    .unwrap();

    // Write the end of the function definition
    writeln!(file, "    ]").unwrap();
    writeln!(file, "}}").unwrap();
}
