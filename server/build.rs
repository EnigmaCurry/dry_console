use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use walkdir::WalkDir;

fn main() {
    // Get the output directory from the environment variable
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("generated_includes.rs");
    let mut file = File::create(dest_path).unwrap();

    // Define the path to the directory containing the snippets
    let dist_path = Path::new("../dist/snippets");

    // Write the start of the function definition to the generated file
    writeln!(
        file,
        "pub fn get_inline_files() -> Vec<(&'static str, &'static [u8], &'static str)> {{"
    )
    .unwrap();
    writeln!(file, "    vec![").unwrap();

    // Walk through the ../dist/snippets directory and find all inline0.js files
    for entry in WalkDir::new(dist_path).into_iter().filter_map(|e| e.ok()) {
        if entry.path().is_file()
            && entry
                .file_name()
                .to_str()
                .map_or(false, |f| f.ends_with(".js"))
        {
            let relative_path = entry.path().strip_prefix("..").unwrap();

            writeln!(
                file,
                "        (\"{}\", include_bytes!(\"../../../../../{}\"), \"application/javascript\"),",
                relative_path
                    .to_string_lossy()
                    .strip_prefix("dist")
                    .unwrap(),
                relative_path.to_string_lossy()
            )
            .unwrap();
        }
    }

    // App CSS
    writeln!(
        file,
        "        (\"/style.css\", include_bytes!(\"../../../../../frontend/style.css\"), \"text/css\"),",
    )
    .unwrap();

    // Patternfly CSS
    // TODO: Tree shake this 1.5MB
    writeln!(
        file,
        "        (\"/patternfly.min.css\", include_bytes!(\"../../../../../frontend/node_modules/@patternfly/patternfly/patternfly.min.css\"), \"text/css\"),",
    )
    .unwrap();
    writeln!(
        file,
        "        (\"/patternfly.min.css.map\", include_bytes!(\"../../../../../frontend/node_modules/@patternfly/patternfly/patternfly.min.css.map\"), \"application/octet-stream\"),",
    )
    .unwrap();

    // Patternfly fonts
    writeln!(
        file,
        "        (\"/assets/fonts/webfonts/fa-solid-900.woff2\", include_bytes!(\"../../../../../frontend/node_modules/@fortawesome/fontawesome-free/webfonts/fa-solid-900.woff2\"), \"font/woff2\"),",
    )
    .unwrap();
    writeln!(
        file,
        "        (\"/assets/fonts/RedHatText/RedHatText-Regular.woff2\", include_bytes!(\"../../../../../frontend/node_modules/@patternfly/patternfly/assets/fonts/RedHatText/RedHatText-Regular.woff2\"), \"font/woff2\"),",
    )
    .unwrap();

    // Write the end of the function definition
    writeln!(file, "    ]").unwrap();
    writeln!(file, "}}").unwrap();
}
