use clap::CommandFactory;
use std::path::PathBuf;

include!("television/cli/args.rs");

/// generate the man page from the Clap configuration
fn build_man_page() -> std::io::Result<()> {
    let out_dir = PathBuf::from(
        std::env::var_os("OUT_DIR").ok_or(std::io::ErrorKind::NotFound)?,
    );
    let cmd = Cli::command();
    let man = clap_mangen::Man::new(cmd);
    let mut buffer = Vec::<u8>::default();
    man.render(&mut buffer)?;

    let out_path = out_dir
        .ancestors()
        .nth(4)
        .unwrap()
        .join("assets")
        .join("tv.1");
    std::fs::create_dir_all(out_path.parent().unwrap())?;
    std::fs::write(&out_path, &buffer)?;
    Ok(())
}

fn main() {
    println!("cargo::rerun-if-changed=television/cli/args.rs");
    println!("cargo::rerun-if-changed=build.rs");
    build_man_page().expect("Failed to generate man page.");
}
