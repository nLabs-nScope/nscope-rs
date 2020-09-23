/***************************************************************************************************
 *
 *  nLabs, LLC
 *  https://nscope.org
 *  Copyright(c) 2020. All Rights Reserved
 *
 *  This file is part of the nScope API
 *
 **************************************************************************************************/

use nscope::{LabBench, Nscope};
use std::{thread, time};

fn main() {
    // Create a LabBench
    let bench = LabBench::new().unwrap();

    // Open all available nScope links
    let nscopes: Vec<Nscope> = bench.list().filter_map(|nsl| nsl.open()).collect();

    loop {
        thread::sleep(time::Duration::from_millis(50));
        for n in nscopes.iter() {
            let data = n.data.read().unwrap();
            println!("{:?} {}", data.power_state, data.power_usage);
        }
    }
}
