/***************************************************************************************************
 *
 *  nLabs, LLC
 *  https://getnlab.com
 *  Copyright(c) 2020. All Rights Reserved
 *
 *  This file is part of the nLab API
 *
 **************************************************************************************************/

use nlabapi::{LabBench};
use std::{thread, time};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    // Create a LabBench
    let bench = LabBench::new()?;

    // Open the first available nLab
    let nlab = bench.open_first_available(true)?;

    nlab.p1.turn_on();
    thread::sleep(time::Duration::from_secs(10));
    nlab.p1.turn_off();

    nlab.p2.turn_on();
    thread::sleep(time::Duration::from_secs(10));
    nlab.p2.turn_off();

    Ok(())
}