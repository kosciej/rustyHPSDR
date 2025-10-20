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

use nix::sys::socket::setsockopt;
use nix::sys::socket::sockopt::{ReuseAddr, ReusePort};
use std::net::UdpSocket;
use std::os::raw::c_int;

use crate::discovery::Device;
use crate::modes::Modes;
use crate::radio::RadioMutex;
use crate::wdsp::*;

const OZY_BUFFER_SIZE: usize = 512;
const METIS_BUFFER_SIZE: usize = (OZY_BUFFER_SIZE * 2) + 8;
const SYNC: u8 = 0x7F;

pub struct Protocol1 {
    device: Device,
    socket: UdpSocket,
    receive_sequence: u32,
    send_sequence: u32,
    wide_sequence: u32,
    receivers: u8,
    current_receiver: u8,
    iq_samples: i32,
    n_samples: i32,
    left_sample: i32,
    right_sample: i32,
    ozy_buffer: Vec<u8>,
    ozy_buffer_offset: usize,
    ozy_command: u8,
    metis_buffer: Vec<u8>,
    metis_buffer_offset: usize,
}

impl Protocol1 {
    pub fn new(device: Device) -> Protocol1 {
        let socket = UdpSocket::bind("0.0.0.0:0").expect("bind failed");
        setsockopt(&socket, ReusePort, &true).unwrap();
        setsockopt(&socket, ReuseAddr, &true).unwrap();

        let receive_sequence: u32 = 0;
        let send_sequence: u32 = 0;
        let wide_sequence: u32 = 0;
        let receivers: u8 = 2;
        let current_receiver: u8 = 0;
        let iq_samples: i32 = (512 - 8) / ((receivers as i32 * 6) + 2);
        let n_samples: i32 = 0;
        let left_sample: i32 = 0;
        let right_sample: i32 = 0;
        let ozy_buffer: Vec<u8> = vec![0; OZY_BUFFER_SIZE];
        let ozy_buffer_offset: usize = 8;
        let ozy_command: u8 = 1;
        let metis_buffer: Vec<u8> = vec![0; METIS_BUFFER_SIZE];
        let metis_buffer_offset: usize = 8;

        Protocol1 {
            device,
            socket,
            receive_sequence,
            send_sequence,
            wide_sequence,
            receivers,
            current_receiver,
            n_samples,
            iq_samples,
            left_sample,
            right_sample,
            ozy_buffer,
            ozy_buffer_offset,
            ozy_command,
            metis_buffer,
            metis_buffer_offset,
        }
    }

    pub fn run(&mut self, radio_mutex: &RadioMutex) {
        // start the radio running
        loop {
            if self.device.device == 6 {
                self.send_ozy_buffer(radio_mutex, 0);
            } else {
                self.send_ozy_buffer(radio_mutex, 0);
            }
            if self.ozy_command == 1 {
                break;
            }
        }
        loop {
            if self.device.device == 6 {
                self.send_ozy_buffer(radio_mutex, 0);
            } else {
                self.send_ozy_buffer(radio_mutex, 0);
            }
            if self.ozy_command == 1 {
                break;
            }
        }

        self.metis_start();

        let mut buffer = vec![0; 2048];
        loop {
            match self.socket.recv_from(&mut buffer) {
                Ok((_size, src)) => {
                    match src.port() {
                        1024 => {
                            if buffer[0] == 0xEF && buffer[1] == 0xFE {
                                let seq = u32::from_be_bytes([
                                    buffer[4], buffer[5], buffer[6], buffer[7],
                                ]);
                                match buffer[2] {
                                    1 => {
                                        match buffer[3] {
                                            6 => {
                                                // IQ samples
                                                self.process_ozy_buffer(&buffer, 8, radio_mutex);
                                                self.process_ozy_buffer(&buffer, 520, radio_mutex);
                                            }
                                            4 => { // Wideband samples
                                            }
                                            _ => println!("Unexpected EP {}", buffer[3]),
                                        }
                                    }
                                    _ => println!("Unexpected packet type {}", buffer[2]),
                                }
                            } else {
                                println!("Received bad header bytes");
                            }
                        }
                        _ => println!("Unknown port {}", src.port()),
                    }
                }
                Err(e) => {
                    eprintln!("Error receiving UDP packet: {}", e);
                }
            }
            let mut r = radio_mutex.radio.lock().unwrap();
            let sample_rate_changed = r.sample_rate_changed;
            r.sample_rate_changed = false;
            drop(r);
            if sample_rate_changed {
                self.metis_stop();
                // receiver change sample rate
                // transmitter change mic sample rate
                self.metis_start();
                // PS set sample rate
            }
        }
    }

    fn process_ozy_buffer(&mut self, buffer: &Vec<u8>, offset: usize, radio_mutex: &RadioMutex) {
        //let mut r = radio.lock().unwrap();
        let mut r = radio_mutex.radio.lock().unwrap();
        let mut audio_buffer: Vec<f64> = vec![0.0; r.receiver[0].output_samples * 2];
        let mic_sample_divisor = r.transmitter.sample_rate / 48000;
        let mut microphone_buffer: Vec<f64> = vec![0.0; r.transmitter.microphone_buffer_size * 2];
        let mut microphone_buffer_offset: usize = 0;
        let mut microphone_iq_buffer: Vec<f64> = vec![0.0; r.transmitter.iq_buffer_size * 2];
        let microphone_iq_buffer_offset: usize = 0;

        let mut i_sample = 0;
        let mut q_sample = 0;
        let mut mic_sample = 0;
        let mut b = offset;
        if buffer[b] == SYNC && buffer[b + 1] == SYNC && buffer[b + 2] == SYNC {
            b += 3;
        } else {
            eprintln!("SYNC error");
            drop(r);
            self.metis_stop();
            loop {
                if self.device.device == 6 {
                    self.send_ozy_buffer(radio_mutex, 0);
                } else {
                    self.send_ozy_buffer(radio_mutex, 0);
                }
                if self.ozy_command == 1 {
                    break;
                }
            }
            self.metis_start();
            return;
        }
        // collect the control bytes
        b += 5;
        for _s in 0..self.iq_samples {
            for rx in 0..r.receivers {
                if buffer[b] & 0x80 != 0 {
                    i_sample =
                        u32::from_be_bytes([0xFF, buffer[b], buffer[b + 1], buffer[b + 2]]) as i32;
                } else {
                    i_sample =
                        u32::from_be_bytes([0, buffer[b], buffer[b + 1], buffer[b + 2]]) as i32;
                }
                b += 3;
                if buffer[b] & 0x80 != 0 {
                    q_sample =
                        u32::from_be_bytes([0xFF, buffer[b], buffer[b + 1], buffer[b + 2]]) as i32;
                } else {
                    q_sample =
                        u32::from_be_bytes([0, buffer[b], buffer[b + 1], buffer[b + 2]]) as i32;
                }
                b += 3;

                let i = r.receiver[rx as usize].samples * 2;
                r.receiver[rx as usize].iq_input_buffer[i] = i_sample as f64 / 16777215.0;
                r.receiver[rx as usize].iq_input_buffer[i + 1] = q_sample as f64 / 16777215.0;
                r.receiver[rx as usize].samples += 1;
                if r.receiver[rx as usize].samples >= r.receiver[rx as usize].buffer_size {
                    let raw_ptr: *mut f64 =
                        r.receiver[rx as usize].iq_input_buffer.as_mut_ptr() as *mut f64;
                    let audio_ptr: *mut f64 = audio_buffer.as_mut_ptr() as *mut f64;
                    let mut result: c_int = 0;
                    unsafe {
                        fexchange0(
                            r.receiver[rx as usize].channel,
                            raw_ptr,
                            audio_ptr,
                            &mut result,
                        );
                    }
                    unsafe {
                        Spectrum0(1, r.receiver[rx as usize].channel, 0, 0, raw_ptr);
                    }
                    r.receiver[rx as usize].samples = 0;
                    if rx == 0 || (rx == 1 && r.rx2_enabled) {
                        for i in 0..r.receiver[rx as usize].output_samples {
                            let ix = i * 2;
                            let left_sample: i32 = (audio_buffer[ix] * 16777215.0) as i32;
                            let right_sample: i32 = (audio_buffer[ix + 1] * 16777215.0) as i32;

                            if r.audio[rx as usize].remote_output {
                                self.ozy_buffer[self.ozy_buffer_offset] = (left_sample >> 8) as u8;
                                self.ozy_buffer_offset += 1;
                                self.ozy_buffer[self.ozy_buffer_offset] = left_sample as u8;
                                self.ozy_buffer_offset += 1;
                                self.ozy_buffer[self.ozy_buffer_offset] = (right_sample >> 8) as u8;
                                self.ozy_buffer_offset += 1;
                                self.ozy_buffer[self.ozy_buffer_offset] = right_sample as u8;
                                self.ozy_buffer_offset += 1;
                            } else {
                                self.ozy_buffer[self.ozy_buffer_offset] = 0;
                                self.ozy_buffer_offset += 1;
                                self.ozy_buffer[self.ozy_buffer_offset] = 0;
                                self.ozy_buffer_offset += 1;
                                self.ozy_buffer[self.ozy_buffer_offset] = 0;
                                self.ozy_buffer_offset += 1;
                                self.ozy_buffer[self.ozy_buffer_offset] = 0;
                                self.ozy_buffer_offset += 1;
                            }

                            // TX iq samples
                            self.ozy_buffer[self.ozy_buffer_offset] = 0;
                            self.ozy_buffer_offset += 1;
                            self.ozy_buffer[self.ozy_buffer_offset] = 0;
                            self.ozy_buffer_offset += 1;
                            self.ozy_buffer[self.ozy_buffer_offset] = 0;
                            self.ozy_buffer_offset += 1;
                            self.ozy_buffer[self.ozy_buffer_offset] = 0;
                            self.ozy_buffer_offset += 1;
                            if self.ozy_buffer_offset == OZY_BUFFER_SIZE {
                                drop(r);
                                if self.device.device == 6 {
                                    self.send_ozy_buffer(radio_mutex, 0);
                                } else {
                                    self.send_ozy_buffer(radio_mutex, 0);
                                }
                                self.ozy_buffer_offset = 8;
                                r = radio_mutex.radio.lock().unwrap();
                            }

                            if r.audio[rx as usize].local_output {
                                let left_sample: i16 = (audio_buffer[ix] * 32767.0) as i16;
                                let right_sample: i16 = (audio_buffer[ix + 1] * 32767.0) as i16;
                                let ox = r.receiver[rx as usize].local_audio_buffer_offset * 2;
                                r.receiver[rx as usize].local_audio_buffer[ox + 1] = left_sample;
                                r.receiver[rx as usize].local_audio_buffer[ox] = right_sample;
                                r.receiver[rx as usize].local_audio_buffer_offset += 1;
                                if r.receiver[rx as usize].local_audio_buffer_offset
                                    == r.receiver[rx as usize].local_audio_buffer_size
                                {
                                    let buffer_clone =
                                        r.receiver[rx as usize].local_audio_buffer.clone();
                                    match r.audio[rx as usize].write_output(&buffer_clone) {
                                        Ok(()) => {}
                                        Err(e) => {
                                            eprintln!("Protocol1: write_output: {}", e);
                                        }
                                    }
                                    r.receiver[rx as usize].local_audio_buffer_offset = 0;
                                }
                            }
                        }
                    }
                }
            }

            // todo process mic samples
            if buffer[b] & 0x80 != 0 {
                mic_sample = u32::from_be_bytes([0xFF, 0x00, buffer[b], buffer[b + 1]]) as i32;
            } else {
                mic_sample = u32::from_be_bytes([0, 0x00, buffer[b], buffer[b + 1]]) as i32;
            }
            let x = microphone_buffer_offset * 2;
            if r.tune {
                microphone_buffer[x] = 0.0;
            } else {
                microphone_buffer[x] = mic_sample as f64 / 32768.0;
            }
            microphone_buffer[x + 1] = 0.0;
            microphone_buffer_offset += 1;
            if microphone_buffer_offset >= r.transmitter.microphone_buffer_size {
                let raw_ptr: *mut f64 = microphone_buffer.as_mut_ptr() as *mut f64;
                let iq_ptr: *mut f64 = microphone_iq_buffer.as_mut_ptr() as *mut f64;
                let mut result: c_int = 0;
                unsafe {
                    fexchange0(r.transmitter.channel, raw_ptr, iq_ptr, &mut result);
                }
                unsafe {
                    Spectrum0(1, r.transmitter.channel, 0, 0, iq_ptr);
                }

                if r.is_transmitting() {
                    for j in 0..r.transmitter.iq_buffer_size {
                        let ix = j * 2;
                        let ox = r.transmitter.iq_samples * 2;
                        r.transmitter.iq_buffer[ox] = microphone_iq_buffer[ix] as f32;
                        r.transmitter.iq_buffer[ox + 1] = microphone_iq_buffer[ix + 1] as f32;
                        r.transmitter.iq_samples += 1;
                        if r.transmitter.iq_samples >= r.transmitter.iq_buffer_size {
                            self.send_ozy_buffer(radio_mutex, 0);
                            r.transmitter.iq_samples = 0;
                        }
                    }
                }

                microphone_buffer_offset = 0;
            }

            b += 2;
        }
    }

    fn send_ozy_buffer(&mut self, radio_mutex: &RadioMutex, rx: i32) {
        let mut c0: u8 = 0x00;
        let mut c1: u8 = 0x00;
        let mut c2: u8 = 0x00;
        let mut c3: u8 = 0x00;
        let mut c4: u8 = 0x00;

        //let r = radio.lock().unwrap();
        let r = radio_mutex.radio.lock().unwrap();
        let mut frequency = r.receiver[0].frequency;
        if r.receiver[0].mode == Modes::CWL.to_usize() {
            frequency += r.receiver[0].cw_pitch;
        } else if r.receiver[0].mode == Modes::CWU.to_usize() {
            frequency -= r.receiver[0].cw_pitch;
        }
        let mut frequency_b = r.receiver[1].frequency;
        if r.receiver[0].mode == Modes::CWL.to_usize() {
            frequency_b += r.receiver[0].cw_pitch;
        } else if r.receiver[0].mode == Modes::CWU.to_usize() {
            frequency_b -= r.receiver[0].cw_pitch;
        }

        let mut attenuation = r.receiver[0].attenuation;
        if r.dev == 6 {
            // Hermes Lite
            attenuation = r.receiver[0].rxgain;
        }

        if self.metis_buffer_offset == 8 {
            c0 = 0x00;
            c2 = 0x00;
            c1 = 0x00;
            match r.receiver[0].sample_rate {
                48000 => {}
                96000 => c1 |= 0x01,
                192000 => c1 |= 0x02,
                384000 => c1 |= 0x03,
                _ => {}
            }
            c2 = 0x00; // TODO Class E and OC
            c3 = 0x00; // TODO adc random, dither, gain, antenna
            match r.adc[r.receiver[0].adc].rx_antenna {
                0 => c3 |= 0x00, // ANT 1
                1 => c3 |= 0x01, // ANT 2
                2 => c3 |= 0x02, // ANT 3
                3 => c3 |= 0x00, // EXT 1
                4 => c3 |= 0x00, // EXT 2
                5 => c3 |= 0x00, // XVTR
                _ => c3 |= 0x00, // None
            }
            c4 = 0x00; // TODO assume using rx antenna for now
            match r.adc[r.receiver[0].adc].rx_antenna {
                0 => c4 |= 0x00,
                1 => c4 |= 0x01,
                2 => c4 |= 0x02,
                _ => c4 |= 0x00,
            }
            c4 |= (self.receivers - 1) << 3;
        } else {
            match self.ozy_command {
                1 => {
                    c0 = 0x02; // C0
                    // TX frequency
                    let mut f: i32 = frequency as i32;
                    if r.split {
                        f = frequency_b as i32;
                    }
                    c1 = (f >> 24) as u8; // C1
                    c2 = (f >> 16) as u8; // C2
                    c3 = (f >> 8) as u8; // C3
                    c4 = f as u8; // C4
                }
                2 => {
                    c0 = 0x04 + (self.current_receiver * 2); // C0
                    // RX frequency
                    let f = r.receiver[self.current_receiver as usize].frequency as i32;
                    c1 = (f >> 24) as u8; // C1
                    c2 = (f >> 16) as u8; // C2
                    c3 = (f >> 8) as u8; // C3
                    c4 = f as u8; // C4
                    self.current_receiver += 1;
                    if self.current_receiver >= self.receivers {
                        self.current_receiver = 0;
                    }
                }
                3 => {
                    c0 = 0x12; // C0
                    c1 = 0x00; // C1
                    c2 = 0x00; // C2
                    if r.mic_boost {
                        c2 |= 0x01;
                    }
                    c3 = 0x00; // C3
                    c4 = 0x00; // C4
                }
                4 => {
                    c0 = 0x14; // C0
                    c1 = 0x00; // C1 // preamp adc 0
                    if r.mic_ptt {
                        c1 |= 0x40;
                    }
                    if r.mic_bias_enable {
                        c1 |= 0x20;
                    }
                    if r.mic_bias_ring {
                        c1 |= 0x10;
                    }
                    c2 = 0x00; // C2
                    c3 = 0x00; // C3
                    c4 = 0x00; // C4
                    if self.device.device == 6 {
                        // HERMES_LITE
                        if self.device.version > 42 {
                            // HERMES_LITE_2
                            c4 |= 0x40;
                            c4 |= ((attenuation + 12) & 0x3F) as u8;
                        } else { // HERMES_LITE_1
                            //c4 |= 0x20;
                        }
                    } else {
                        //if attenuation != 0 {
                        c4 = 0x20;
                        c4 |= (attenuation & 0x1F) as u8;
                        //}
                    }
                }
                5 => {
                    c0 = 0x16; // C0
                    c1 = 0x00; // C1
                    c2 = 0x00; // C2
                    c3 = 0x0C; // C3
                    c4 = 0x1E; // C4
                }
                6 => {
                    c0 = 0x1C; // C0
                    c1 = 0x00; // C1
                    c2 = 0x00; // C2
                    c3 = 0x00; // C3
                    c4 = 0x00; // C4
                }
                7 => {
                    c0 = 0x1E; // C0
                    c1 = 0x00; // C1
                    c2 = 0x14; // C2
                    c3 = 0x14; // C3
                    c4 = 0x00; // C4
                }
                8 => {
                    c0 = 0x20; // C0
                    c1 = 0x00; // C1
                    c2 = 0x00; // C2
                    c3 = 0x28; // C3
                    c4 = 0x0A; // C4
                }
                9 => {
                    c0 = 0x22; // C0
                    c1 = 0x19; // C1
                    c2 = 0x00; // C2
                    c3 = 0xC8; // C3
                    c4 = 0x00; // C4
                }
                10 => {
                    c0 = 0x24; // C0
                    c1 = 0x00; // C1
                    c2 = 0x00; // C2
                    c3 = 0x00; // C3
                    c4 = 0x00; // C4
                }
                11 => {
                    c0 = 0x2E; // C0
                    c1 = 0x00; // C1
                    c2 = 0x00; // C2
                    c3 = 0x04; // C3
                    c4 = 0x15; // C4
                }
                _ => eprintln!("Invalid command {}", self.ozy_command),
            }
            if self.current_receiver == 0 {
                self.ozy_command += 1;
                if self.ozy_command > 11 {
                    self.ozy_command = 1;
                }
            }
        }

        if r.is_transmitting() {
            c0 |= 0x01;
        }

        self.ozy_buffer[0] = SYNC;
        self.ozy_buffer[1] = SYNC;
        self.ozy_buffer[2] = SYNC;
        self.ozy_buffer[3] = c0;
        self.ozy_buffer[4] = c1;
        self.ozy_buffer[5] = c2;
        self.ozy_buffer[6] = c3;
        self.ozy_buffer[7] = c4;

        self.metis_write();
    }

    fn metis_write(&mut self) {
        // copy the buffer
        for i in 0..512 {
            self.metis_buffer[i + self.metis_buffer_offset] = self.ozy_buffer[i];
        }

        if self.metis_buffer_offset == 8 {
            self.metis_buffer_offset = 520;
        } else {
            self.metis_buffer[0] = 0xEF;
            self.metis_buffer[1] = 0xFE;
            self.metis_buffer[2] = 0x01;
            self.metis_buffer[3] = 0x02;
            self.metis_buffer[4] = (self.send_sequence >> 24) as u8;
            self.metis_buffer[5] = (self.send_sequence >> 16) as u8;
            self.metis_buffer[6] = (self.send_sequence >> 8) as u8;
            self.metis_buffer[7] = self.send_sequence as u8;
            self.socket
                .send_to(&self.metis_buffer, self.device.address)
                .expect("couldn't send data");
            self.send_sequence += 1;
            self.metis_buffer_offset = 8;
        }
    }

    fn metis_start(&self) {
        let mut buf = [0u8; 64];
        buf[0] = 0xEF;
        buf[1] = 0xFE;
        buf[2] = 0x04;
        buf[3] = 0x03;
        self.socket
            .send_to(&buf, self.device.address)
            .expect("couldn't send data");
    }

    fn metis_stop(&self) {
        let mut buf = [0u8; 64];
        buf[0] = 0xEF;
        buf[1] = 0xFE;
        buf[2] = 0x04;
        buf[3] = 0x00;
        self.socket
            .send_to(&buf, self.device.address)
            .expect("couldn't send data");
    }

    fn f64_to_f32(input: Vec<f64>) -> Vec<f32> {
        input.into_iter().map(|x| x as f32).collect()
    }
}
