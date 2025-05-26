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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transmitter {
    pub channel: i32,
    pub sample_rate: i32,
    pub dsp_rate: i32,
    pub output_rate: i32,
    pub buffer_size: i32,
    pub output_samples: i32,
    pub p1_packet_size: i32,
    pub packet_counter: i32,
    pub is_transmitting: bool,
}

impl Transmitter {

    pub fn new(chan: u8, protocol: u8 ) -> Transmitter {

        let channel: i32 = chan as i32;
        let sample_rate = 48000; // protocol 1 & 2
        let mut dsp_rate = 48000;    // protocol 1
        let mut output_rate = 48000; // protocol 1
        if protocol == 2 {
            dsp_rate = 96000;
            output_rate = 192000;
        }
        let buffer_size=1024;
        let mut output_samples=1024;
        let p1_packet_size = 126;
        let packet_counter = 0;
        if protocol == 2 {
            output_samples = 1024*(output_rate/sample_rate);
        }

        let is_transmitting = false;


        let tx = Transmitter{ channel, sample_rate, dsp_rate, output_rate, buffer_size, output_samples, p1_packet_size, packet_counter, is_transmitting };

        tx
    }

    pub fn init(&mut self) {
    }

    pub fn run(&mut self) {
    }

}
