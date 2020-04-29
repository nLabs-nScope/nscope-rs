use nscope;

fn main() -> Result<(), nscope::NscopeError> {
    let mut bench = nscope::LabBench::new()?;
    bench.refresh();
    print!("{:#?}", bench.nscopes());
    bench.open("my awesome nScope")?;
    print!("{:#?}", bench.nscopes());
    bench.open_one(0, "my awesome nScope 2")?;
    print!("{:#?}", bench.nscopes());
    Ok(())
}
