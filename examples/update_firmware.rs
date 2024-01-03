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
    println!("{:?}", bench);

    // Update any scope that is in DFU mode
    for nscope_link in bench.list() {
        if let Err(e) =  nscope_link.update() {
            println!("Encountered an error updating nScope: {e}")
        }
    }

    Ok(())
}
