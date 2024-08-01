use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

fn main() -> io::Result<()> {
    println!("hmm");
    let project_root: String = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .parent()
        .unwrap()
        .as_os_str()
        .to_string_lossy()
        .to_string();
    // Define the source and destination paths
    let files_to_copy = vec![
        (
            "node_modules/@patternfly/patternfly/patternfly.min.css",
            format!("{project_root}/dist/patternfly.min.css"),
        ),
        (
            "node_modules/@patternfly/patternfly/patternfly.min.css.map",
            format!("{project_root}/dist/patternfly.min.css.map"),
        ),
        (
            "node_modules/@patternfly/patternfly/assets/fonts/RedHatText/RedHatText-Regular.woff2",
            format!("{project_root}/dist/assets/fonts/RedHatText/RedHatText-Regular.woff2"),
        ),
        (
            "node_modules/@fortawesome/fontawesome-free/webfonts/fa-solid-900.woff2",
            format!("{project_root}/dist/assets/fonts/webfonts/fa-solid-900.woff2"),
        ),
    ];

    // Iterate over the files and copy them
    for (src, dest) in files_to_copy {
        // Create the destination directory if it doesn't exist
        if let Some(parent) = Path::new(&dest).parent() {
            fs::create_dir_all(parent)?;
        }

        // Copy the file to the destination
        dbg!(dest.clone());
        fs::copy(src, dest)?;
    }

    Ok(())
}
