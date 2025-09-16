use std::{fs, process::Command};

/// Custom build script for mrkt that automatically watches for changes in files
/// and triggers the Tailwind CSS build process.
fn main() {
    let tailwind_cmd = "npx tailwindcss -i src/site/index.css -o static/site.css";

    let path = fs::canonicalize(".");

    let path = match path {
        Ok(p) => p,
        Err(e) => panic!("{}", e),
    };

    // Assuming we're running on a Unix-like system with bash available, if you
    // need to run this on Windows, you'll probably need to adjust this to be
    // `cmd` or `powershell` and change the arguments accordingly.
    Command::new("bash")
        .current_dir(path)
        .arg("-c")
        .arg(tailwind_cmd)
        .spawn()
        .expect("Failed running tailwind");
    println!("cargo:warning=Finished running tailwind");
}
