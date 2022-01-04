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
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    // Create a LabBench
    let bench = LabBench::new()?;

    // Open all available nScope links
    let mut nscopes: Vec<Nscope> = bench.open_all_available();

    loop {
        thread::sleep(time::Duration::from_millis(10));

        nscopes.retain(|n| n.is_connected());

        for n in nscopes.iter() {
            match n.power_status() {
                Ok(status) => {
                    let state = format!("{:?}", status.state);
                    println!("{:>15}: {:.3} Watts", state, status.usage)
                },
                Err(error) => eprintln!("{}", error),
            }
        }

        if nscopes.is_empty() {
            return Err("Cannot find any nScopes")?
        }
    }
}
