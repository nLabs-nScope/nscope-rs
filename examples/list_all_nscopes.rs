use nscope::LabBench;

fn main() {

    // Create a LabBench
    let bench = LabBench::new().unwrap();


    println!("{:?}", bench);

    for i in bench.list() {
        println!("{:?}", i)
    }

}