/***************************************************************************************************
 *
 *  nLabs, LLC
 *  https://nscope.org
 *  Copyright(c) 2020. All Rights Reserved
 *
 *  This file is part of the nScope API
 *
 **************************************************************************************************/

pub(crate) static FIRMWARE: &[u8] = include_bytes!("firmware/v2");
pub(crate) static FIRMWARE_VERSION: u16 = 0x0201;