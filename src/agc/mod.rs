/*
    Copyright (C) 2025  John Melton G0ORX/N6LYT

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

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

