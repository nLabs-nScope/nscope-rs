/***************************************************************************************************
 *
 *  nLabs, LLC
 *  https://nscope.org
 *  Copyright(c) 2020. All Rights Reserved
 *
 *  This file is part of the nScope API
 *
 **************************************************************************************************/

use nscope::{AnalogSignalPolarity, LabBench, Trigger, TriggerType};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    // Create a LabBench
    let bench = LabBench::new()?;

    // Open the first available nScope
    let nscope = bench.open_first_available(true)?;

    nscope.a1.turn_on();

    let rq = nscope.request(4.0,20, None);

    for sample in rq.receiver {
        println!("{:?}", sample.data);
    }

    nscope.a1.turn_off();
    
    
    let rq = nscope.request(4.0, 20, Some(Trigger{
        is_enabled: true,
        trigger_type: TriggerType::RisingEdge,
        source_channel: 0,
        trigger_level: 0.0,
        trigger_delay_us: 0,
    }));

    nscope.a1.set_polarity(AnalogSignalPolarity::Bipolar);
    nscope.a1.turn_on();
    for sample in rq.receiver {
        println!("{:?}", sample.data);
    }

    Ok(())
}