/***************************************************************************************************
 *
 *  nLabs, LLC
 *  https://nscope.org
 *  Copyright(c) 2020. All Rights Reserved
 *
 *  This file is part of the nScope API
 *
 **************************************************************************************************/

use std::{thread, time};
use nscope::{LabBench, Nscope};

fn main() {

    // Create a LabBench
    let bench = nscope::LabBench::new().unwrap();

    // Open all available nScope links
    let nscopes: Vec<Nscope> = bench.list().filter_map(|nsl| nsl.open()).collect();


}