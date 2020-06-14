fn main() {
    let bench = nscope::LabBench::new().unwrap();

    // You can debug print the entire bench
    println!("{:#?}", bench);

    // You can also print just the list of nScopes
    println!("{:#?}", bench.nscopes());
}
