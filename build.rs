use anyhow::Result;
use vergen_gix::{BuildBuilder, CargoBuilder, Emitter, RustcBuilder};

fn main() -> Result<()> {
    let build = BuildBuilder::default().build_date(true).build()?;
    let cargo = CargoBuilder::default().target_triple(true).build()?;
    let rustc = RustcBuilder::default().semver(true).build()?;
    Emitter::default()
        .add_instructions(&build)?
        .add_instructions(&cargo)?
        .add_instructions(&rustc)?
        .emit()
}
