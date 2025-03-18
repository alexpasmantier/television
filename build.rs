use clap::CommandFactory;
use std::path::PathBuf;

include!("television/cli/args.rs");

/// generate the man page from the Clap configuration
fn build_man_page() -> std::io::Result<()> {
    let out_dir = PathBuf::from(
        std::env::var_os("OUT_DIR").ok_or(std::io::ErrorKind::NotFound)?,
    );
    std::fs::create_dir_all(&out_dir)?;
    let cmd = Cli::command();
    let man = clap_mangen::Man::new(cmd);
    let mut buffer = Vec::<u8>::default();
    man.render(&mut buffer)?;

    let out_path = out_dir.join("tv.1");
    std::fs::write(&out_path, &buffer)?;
    eprintln!("Wrote man page to {out_path:?}");
    std::fs::write(PathBuf::from("./man").join("tv.1"), &buffer)?;
    eprintln!("Wrote man page to ./man directory.");
    Ok(())
}

fn main() {
    println!("cargo::rerun-if-changed=television/cli/args.rs");
    build_man_page().expect("Failed to generate man page.");
}
