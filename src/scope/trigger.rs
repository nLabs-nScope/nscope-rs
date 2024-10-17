/***************************************************************************************************
 *
 *  nLabs, LLC
 *  https://getnlab.com
 *  Copyright(c) 2020. All Rights Reserved
 *
 *  This file is part of the nLab API
 *
 **************************************************************************************************/

/// Different trigger types used to start a data sweep
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum TriggerType {
    RisingEdge,
    FallingEdge,
}

/// A representation of a trigger used to start a data sweep
#[derive(Debug, Copy, Clone)]
pub struct Trigger {
    pub is_enabled: bool,
    pub trigger_type: TriggerType,
    pub source_channel: usize,
    pub trigger_level: f64,
    pub trigger_delay_us: u32,
}

impl Default for Trigger {
    fn default() -> Self {
        Trigger {
            is_enabled: false,
            trigger_type: TriggerType::RisingEdge,
            source_channel: 0,
            trigger_level: 0.0,
            trigger_delay_us: 0,
        }
    }
}

impl TriggerType {
    pub(crate) fn value(&self) -> u8 {
        match self {
            TriggerType::RisingEdge => { 2 }
            TriggerType::FallingEdge => { 1 }
        }
    }
}