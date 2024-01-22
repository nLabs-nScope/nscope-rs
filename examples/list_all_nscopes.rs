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
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    // Create a LabBench
    let bench = LabBench::new()?;

    // Print the bench to show a list of detected nScopes
    println!("Lab Bench: \n{:?}", bench);

    println!("\nManual list of all detected nScopes:");
    // Or loop over all nScope links in the list and print them
    for nscope_link in bench.list() {
        println!("    {:?}", nscope_link)
    }
    Ok(())
}
