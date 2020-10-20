/***************************************************************************************************
 *
 *  nLabs, LLC
 *  https://nscope.org
 *  Copyright(c) 2020. All Rights Reserved
 *
 *  This file is part of the nScope API
 *
 **************************************************************************************************/

use nscope::LabBench;

fn main() {
    env_logger::init();

    // Create a LabBench
    let bench = LabBench::new().unwrap();

    // Print the bench to show a list of connected nScopes
    println!("{:?}", bench);

    // Or loop over all nScope links in the list and print them
    for nsl in bench.list() {
        println!("{:?}", nsl)
    }
}
