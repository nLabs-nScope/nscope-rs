use nscope;

fn main() {
    let mut bench = nscope::LabBench::new().expect("Failed to initialize nscope::Context");
    bench.refresh();
    println!("{:?}", bench);
    bench.open().expect("Unable to open nScope");
    println!("{:?}", bench);
}
