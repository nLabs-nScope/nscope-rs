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
    let mut nscopes: Vec<Nscope> = bench.list().filter_map(|nsl| nsl.open()).collect();

    loop {
        thread::sleep(time::Duration::from_millis(50));

        nscopes.retain(|n| n.is_connected());

        for n in nscopes.iter() {
            match n.power_status() {
                Ok(status) => println!("{:?} {}", status.state, status.usage),
                Err(error) => println!("{}", error),
            }
        }

        if nscopes.is_empty() {
            eprintln!("Cannot find any nScopes");
            break;
        }
    }
}
