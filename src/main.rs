use nscope;

fn main() {
    let mut bench = nscope::LabBench::new().expect("Failed to initialize nscope::Context");
    bench.refresh();
    println!("{:?}", bench);
}
