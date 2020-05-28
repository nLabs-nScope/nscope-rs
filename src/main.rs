fn main() -> Result<(), nscope::NscopeError> {
    println!("{:#?}", nscope::ver());

    let mut bench = nscope::LabBench::new()?;
    bench.refresh();
    println!("{:#?}", bench.nscopes());

    bench.open("my awesome nScope")?;
    println!("{:#?}", bench.nscopes());

    bench.open_one(0, "my awesome nScope 2")?;
    println!("{:#?}", bench.nscopes());

    Ok(())
}
