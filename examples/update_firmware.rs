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
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    // Create a LabBench
    let mut bench = LabBench::new()?;

    // Print the bench to show a list of detected nScopes
    println!("{:?}", bench);

    for nscope_link in bench.list() {
        // Request DFU on any nScope that is available
        if nscope_link.available {
            if let Err(e) = nscope_link.request_dfu() {
                println!("Failed to request DFU on an available nScope: {e}")
            }
        }
    }

    // Wait 500ms for the scope to detach and re-attach as DFU
    thread::sleep(Duration::from_millis(500));
    bench.refresh();

    // Print the bench to show the refreshed list
    println!("{:?}", bench);

    for nscope_link in bench.list() {
        if let Err(e) = nscope_link.update() {
            println!("Encountered an error updating nScope: {e}")
        }
    }

    Ok(())
}
