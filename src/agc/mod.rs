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

use crate::receiver::Receiver;
use crate::wdsp::*;
use serde::{Deserialize, Serialize};

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

    pub fn set_agc(rx: &Receiver, channel: i32) {
        unsafe {
            SetRXAAGCMode(channel, rx.agc.to_i32());
            SetRXAAGCSlope(channel, rx.agcslope);
            SetRXAAGCTop(channel, rx.agcgain.into());
        }
        match rx.agc {
            AGC::OFF => {
                // nothing else to do as mode already set to OFF
            }
            AGC::LONG => unsafe {
                SetRXAAGCAttack(channel, 2);
                SetRXAAGCHang(channel, 2000);
                SetRXAAGCDecay(channel, 2000);
                SetRXAAGCHangThreshold(channel, rx.agcchangethreshold);
            },
            AGC::SLOW => unsafe {
                SetRXAAGCAttack(channel, 2);
                SetRXAAGCHang(channel, 1000);
                SetRXAAGCDecay(channel, 500);
                SetRXAAGCHangThreshold(channel, rx.agcchangethreshold);
            },
            AGC::MEDIUM => unsafe {
                SetRXAAGCAttack(channel, 2);
                SetRXAAGCHang(channel, 0);
                SetRXAAGCDecay(channel, 250);
                SetRXAAGCHangThreshold(channel, 100);
            },
            AGC::FAST => unsafe {
                SetRXAAGCAttack(channel, 2);
                SetRXAAGCHang(channel, 0);
                SetRXAAGCDecay(channel, 50);
                SetRXAAGCHangThreshold(channel, 100);
            },
        }
    }
}
