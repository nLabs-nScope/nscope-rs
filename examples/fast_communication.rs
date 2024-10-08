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

    // Open the first available nScope
    let mut nscope = bench.open_first_available(true)?;

    nscope.ch1.turn_on();
    nscope.ch2.turn_on();
    nscope.ch3.turn_on();
    nscope.ch4.turn_on();

    loop {
        let sweep_handle = nscope.request(2_000_000.0, 1200, None);
        while sweep_handle.receiver.recv().is_ok() {}
    }
}