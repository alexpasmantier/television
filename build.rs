fn main() -> Result<(), Box<dyn std::error::Error>> {
    let build = vergen::BuildBuilder::all_build()?;
    let cargo = vergen::CargoBuilder::all_cargo()?;
    let rustc = vergen::RustcBuilder::all_rustc()?;

    vergen::Emitter::default()
        .add_instructions(&build)?
        .add_instructions(&cargo)?
        .add_instructions(&rustc)?
        .emit()?;

    Ok(())
}
