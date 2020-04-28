use nscope;

fn main() {
    let mut bench = nscope::LabBench::new().expect("Failed to initialize nscope::Context");
    bench.refresh();
    bench.open();
    println!("{:?}", bench);
}
