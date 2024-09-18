use convert_case::Casing;
use dry_console_common::token::generate_deterministic_ulid_from_seed;
use std::env;
use std::fs;
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

    // Walk through the ../dist/snippets directory and find all javascript files
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

    include_shell_scripts(out_dir, project_root);
}

fn include_shell_scripts(out_dir: String, project_root: String) {
    let dest_path = Path::new(&out_dir).join("generated_command_library.rs");

    let mut script_dir = Path::new(&project_root).join("server/src/api/workstation/scripts");
    if let Ok(d) = script_dir.canonicalize() {
        script_dir = d;
    } else {
        panic!("Could not find script directory.");
    }

    let mut output = String::new();

    // Add necessary imports
    output.push_str("use std::collections::HashMap;\n");
    output.push_str("use crate::api::workstation::command::CommandLibraryItem;\n");
    output.push_str("use lazy_static::lazy_static;\n\n");

    // Start of the static HashMap declaration
    output.push_str("lazy_static! {\n");
    output.push_str(
        "    pub static ref STATIC_COMMAND_LIBRARY_MAP: HashMap<String, CommandLibraryItem> = {\n",
    );
    output.push_str("        let mut m = HashMap::new();\n");

    let mut found_variants = std::collections::HashSet::new();

    for entry in fs::read_dir(&script_dir).expect("Failed to read script directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("sh") {
            if let Some(file_name) = path.file_stem().and_then(|s| s.to_str()) {
                let variant_name = file_name.to_case(convert_case::Case::Pascal);
                found_variants.insert(variant_name.clone());

                let script_content = fs::read_to_string(&path).expect("Failed to read script file");

                // Generate ULID from the script content
                let ulid = generate_deterministic_ulid_from_seed(&script_content);

                // Add the entry to the static HashMap using the string representation of the ULID
                output.push_str(&format!(
                    "        m.insert(\"{}\".to_string(), CommandLibraryItem::{});\n",
                    ulid.to_string(),
                    variant_name
                ));
            }
        }
    }

    // End of the static HashMap declaration
    output.push_str("        m\n");
    output.push_str("    };\n");
    output.push_str("}\n\n");

    // Now, generate the CommandLibraryItem implementation with get_script method
    output.push_str("impl CommandLibraryItem {\n");

    // Modified get_script method with the command_id and command_script arguments
    output.push_str(
        "    fn get_script(&self, command_id: &HashMap<CommandLibraryItem, String>, command_script: &HashMap<String, String>) -> String {\n",
    );
    output.push_str("        if let Some(ulid) = command_id.get(self) {\n");
    output.push_str("            if let Some(script) = command_script.get(ulid) {\n");
    output.push_str("                return script.clone();\n");
    output.push_str("            }\n");
    output.push_str("        }\n");

    // Existing logic for getting script content
    output.push_str("        match self {\n");

    for variant_name in found_variants.iter() {
        let file_path = script_dir.join(format!(
            "{}.sh",
            variant_name.to_case(convert_case::Case::Snake)
        ));
        output.push_str(&format!(
            "            CommandLibraryItem::{variant_name} => include_str!(\"{}\").to_string(),\n",
            file_path.to_str().unwrap(),
        ));
    }

    output.push_str("        }\n");
    output.push_str("    }\n");

    output.push_str("}\n");

    fs::write(dest_path, output).expect("Failed to write generated file");
}
