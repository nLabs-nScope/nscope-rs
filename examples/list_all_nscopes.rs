fn main() {

    // Get a new LabBench
    let bench = nscope::LabBench::new();

    // Get the list of available scopes by index
    let available = bench.available_scopes();
    // Print the number of available scopes
    println!("Available nScopes: {:}", available.len());


    // You can also debug print the entire bench to see all scopes
    println!("{:?}", bench);
}
