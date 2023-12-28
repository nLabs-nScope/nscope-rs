/***************************************************************************************************
 *
 *  nLabs, LLC
 *  https://nscope.org
 *  Copyright(c) 2020. All Rights Reserved
 *
 *  This file is part of the nScope API
 *
 **************************************************************************************************/

use nscope::{LabBench, NscopeDFU};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    // Create a LabBench
    let bench = LabBench::new()?;

    // Print the bench to show a list of detected nScopes
    println!("{:?}", bench);

    // Get a list of all the nScopes that are detected in DFU mode
    let scopes_in_dfu: Vec<NscopeDFU> = bench.scopes_in_dfu().collect();

    // If we have a scope in DFU mode, update the first one we found
    if let Some(dfu) = scopes_in_dfu.first() {
        dfu.update()?;
    } else {
        println!("Cannot find any nScopes in DFU mode");
    }

    Ok(())
}
