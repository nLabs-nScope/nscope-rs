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
    let nscope = bench.open_first_available()?;

    nscope.a1.turn_on();

    let rq = nscope.request(4.0,20);

    for sample in rq.receiver {
        println!("{:?}", sample.data);
    }

    nscope.a1.turn_off();

    Ok(())
}