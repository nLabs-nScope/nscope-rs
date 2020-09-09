fn main() {

    // Get a new LabBench
    let bench = nscope::LabBench::new();

    // Print the number of detected scopes on the bench
    println!("Detected nScopes: {:}", bench.nscopes.len());

    // You can also debug print the entire bench to see all scopes
    println!("{:?}", bench);

    {
        // Create a vector to hold our borrowed nscopes
        let mut nscopes = Vec::new();

        // Check out every available nScope
        for scope in bench.nscopes.iter() {
            match scope.checkout() {
                Ok(s) => nscopes.push(s),
                Err(_) => ()
            }
        }

        // Print the bench and note that all are checked out
        println!("{:?}", bench);

        // Pop the first scope off and note that it
        if let Some(nscope) = nscopes.pop() {
            nscope.checkin();
        }

        // Print the bench and note that all are checked out
        println!("{:?}", bench);
        for scope in nscopes.iter() {
            println!("{:?}", scope)
        }

        // When the borrowed scopes go out of scope, they are returned to the bench
    }

    // Print the bench and all should be returned
    println!("{:?}", bench);
}
