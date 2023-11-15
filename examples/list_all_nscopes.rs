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

    // Print the bench to show a list of detected nScopes
    println!("{:?}", bench);

    {
        let mut foo = bench.open_first_available(false).unwrap();
        println!("{:?}", bench);

        {
            let mut foo2 = bench.open_first_available(false).unwrap();
            println!("{:?}", bench);
            foo2.close();
        }

        println!("{:?}", bench);
        foo.close();
    }

    println!("{:?}", bench);

    // // Or loop over all nScope links in the list and print them
    // for nscope_link in bench.list() {
    //     println!("{:?}", nscope_link)
    // }
    Ok(())
}
