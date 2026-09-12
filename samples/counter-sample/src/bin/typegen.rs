fn main() -> anyhow::Result<()> {
    counter_sample::actions::force_link();
    let dest = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("pi/src/generated/contracts.ts");
    streamdeck_plugin::export_typescript(dest)?;
    Ok(())
}
