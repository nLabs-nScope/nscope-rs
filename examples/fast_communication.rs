/***************************************************************************************************
 *
 *  nLabs, LLC
 *  https://getnlab.com
 *  Copyright(c) 2020. All Rights Reserved
 *
 *  This file is part of the nLab API
 *
 **************************************************************************************************/

use nlabapi::LabBench;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    // Create a LabBench
    let bench = LabBench::new()?;

    // Open the first available nLab
    let mut nlab = bench.open_first_available(true)?;

    nlab.ch1.turn_on();
    nlab.ch2.turn_on();
    nlab.ch3.turn_on();
    nlab.ch4.turn_on();

    loop {
        let sweep_handle = nlab.request(2_000_000.0, 1200, None);
        while sweep_handle.receiver.recv().is_ok() {}
    }
}