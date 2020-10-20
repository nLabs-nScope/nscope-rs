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

    nscopes[0].set_ax_on(1, true);

    thread::sleep(time::Duration::from_secs(10));

    nscopes[0].set_ax_on(1, false);
    nscopes[0].set_ax_on(0, true);

    thread::sleep(time::Duration::from_secs(10));

    nscopes[0].set_ax_on(0, false);
}
