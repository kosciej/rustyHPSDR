use serde::{Deserialize, Serialize};
use crate::receiver::Receiver;
use crate::wdsp::*;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum AGC {
    OFF = 0,
    LONG,
    SLOW,
    MEDIUM,
    FAST,
}

impl AGC {
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(AGC::OFF),
            1 => Some(AGC::LONG),
            2 => Some(AGC::SLOW),
            3 => Some(AGC::MEDIUM),
            4 => Some(AGC::FAST),
            _ => None,
        }
    }

    pub fn to_i32(&self) -> i32 {
        *self as i32
    }

    pub fn set_agc(rx: &Receiver) {
        unsafe {
            SetRXAAGCMode(rx.channel, rx.agc.to_i32());
            SetRXAAGCSlope(rx.channel,rx.agcslope);
            SetRXAAGCTop(rx.channel,rx.agcgain.into());
        }
        match rx.agc {
            AGC::OFF => {
                       // nothing else to do as mode already set to OFF
                },
            AGC::LONG => {
                unsafe {
                    SetRXAAGCAttack(rx.channel,2);
                    SetRXAAGCHang(rx.channel,2000);
                    SetRXAAGCDecay(rx.channel,2000);
                    SetRXAAGCHangThreshold(rx.channel,rx.agcchangethreshold);
                }
                },
            AGC::SLOW => {
                unsafe {
                    SetRXAAGCAttack(rx.channel,2);
                    SetRXAAGCHang(rx.channel,1000);
                    SetRXAAGCDecay(rx.channel,500);
                    SetRXAAGCHangThreshold(rx.channel,rx.agcchangethreshold);
                }
                },
            AGC::MEDIUM => {
                unsafe {
                    SetRXAAGCAttack(rx.channel,2);
                    SetRXAAGCHang(rx.channel,0);
                    SetRXAAGCDecay(rx.channel,250);
                    SetRXAAGCHangThreshold(rx.channel,100);
                }
                },
            AGC::FAST => {
                unsafe {
                    SetRXAAGCAttack(rx.channel,2);
                    SetRXAAGCHang(rx.channel,0);
                    SetRXAAGCDecay(rx.channel,50);
                    SetRXAAGCHangThreshold(rx.channel,100);
                }
                },
        }
    }

}

