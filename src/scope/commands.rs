/***************************************************************************************************
 *
 *  nLabs, LLC
 *  https://nscope.org
 *  Copyright(c) 2020. All Rights Reserved
 *
 *  This file is part of the nScope API
 *
 **************************************************************************************************/

pub(super) const NULL_REQ: [u8; 2] = [0, 0xFF];

#[derive(Copy, Clone)]
pub enum Command {
    Quit,
}
