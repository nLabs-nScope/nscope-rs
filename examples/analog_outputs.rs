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

    println!("Querying nScope: {:?}", nscopes[0].get_ax());

    nscopes[0].set_ax_on(true);

    println!("Querying nScope: {:?}", nscopes[0].get_ax());

    thread::sleep(time::Duration::from_secs(10));

    nscopes[0].set_ax_on(false);

    println!("Querying nScope: {:?}", nscopes[0].get_ax());
}
