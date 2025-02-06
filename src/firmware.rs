/***************************************************************************************************
 *
 *  nLabs, LLC
 *  https://getnlab.com
 *  Copyright(c) 2020. All Rights Reserved
 *
 *  This file is part of the nLab API
 *
 **************************************************************************************************/

pub(crate) static FIRMWARE: &[u8] = include_bytes!("firmware/v2");
pub(crate) static FIRMWARE_VERSION: u16 = 0x0206;