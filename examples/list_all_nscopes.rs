fn main() {
    let bench = nscope::LabBench::new().unwrap();
    println!("{:#?}", bench.nscopes());
}
