/***************************************************************************************************
 *
 *  nLabs, LLC
 *  https://nscope.org
 *  Copyright(c) 2020. All Rights Reserved
 *
 *  This file is part of the nScope API
 *
 **************************************************************************************************/

use nscope::{LabBench};
use std::{thread, time};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    // Create a LabBench
    let bench = LabBench::new()?;

    // Open the first available nScope
    let nscope = bench.open_first_available()?;

    nscope.a1.turn_on();
    thread::sleep(time::Duration::from_secs(10));
    nscope.a1.turn_off();

    nscope.a2.turn_on();
    thread::sleep(time::Duration::from_secs(10));
    nscope.a2.turn_off();

    Ok(())
}