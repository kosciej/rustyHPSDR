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
use std::net::{UdpSocket};
use std::os::raw::c_int;

use crate::alex::*;
use crate::discovery::Device;
use crate::modes::Modes;
use crate::receiver::Receiver;
use crate::radio::{Keyer, RadioMutex};
use crate::wdsp::*;

const HEADER_SIZE: usize  = 16;  // 16 byte header
const SAMPLE_SIZE: usize = 3;    // 3 byte (24 bit) samples
const INTERLEAVE_FACTOR: usize = 2; // 2 samples (I & Q) interleaved
const MIC_HEADER_SIZE: usize = 4;   // just a sequance number
const MIC_SAMPLE_SIZE: usize = 2;    // 2 byte (16 bit) samples
const MIC_SAMPLES: usize = 64;       // 64 samples per buffer
const IQ_BUFFER_SIZE: usize = 240;   // 240 IQ samples

#[derive(Debug)]
pub struct Protocol2 {
    device: Device, 
    socket: UdpSocket,     
    general_sequence: u32,
    high_priority_sequence: u32,
    receive_specific_sequence: u32,
    transmit_specific_sequence: u32,
    audio_sequence: u32,
    tx_iq_sequence: u32,
}   

impl Protocol2 {

    pub fn new(device: Device) -> Protocol2 {
        let socket = UdpSocket::bind("0.0.0.0:0").expect("bind failed");
        setsockopt(&socket, ReusePort, &true).unwrap();
        setsockopt(&socket, ReuseAddr, &true).unwrap();

        let general_sequence: u32 = 0;
        let high_priority_sequence: u32 = 0;
        let receive_specific_sequence: u32 = 0;
        let transmit_specific_sequence: u32 = 0; 
        let audio_sequence: u32 = 0; 
        let tx_iq_sequence: u32 = 0; 

        let p2 = Protocol2{device,
                           socket,
                           general_sequence,
                           high_priority_sequence,
                           receive_specific_sequence,
                           transmit_specific_sequence,
                           audio_sequence,
                           tx_iq_sequence,
        };

        p2

    }

    pub fn run(&mut self, radio_mutex: &RadioMutex) {
        let r = radio_mutex.radio.lock().unwrap();
        let mut buffer = vec![0; 65536];
        let mut audio_buffer: Vec<f64> = vec![0.0; (r.receiver[0].output_samples*2) as usize];
        let mut microphone_buffer: Vec<f64> = vec![0.0; (r.transmitter.microphone_buffer_size * 2) as usize];
        let mut microphone_buffer_offset: usize = 0;
        let mut microphone_iq_buffer: Vec<f64> = vec![0.0; (r.transmitter.iq_buffer_size * 2) as usize];
        let mut microphone_iq_buffer_offset: usize = 0;
        let mut tx_iq_buffer: Vec<f64> = vec![0.0; IQ_BUFFER_SIZE*2];
        let mut tx_iq_buffer_offset: usize = 0;
        drop(r);

        self.send_general();
        self.send_high_priority(radio_mutex);
        self.send_transmit_specific(radio_mutex);
        self.send_receive_specific(radio_mutex);


        loop {
            match self.socket.recv_from(&mut buffer) {
                Ok((size, src)) => {
                    match src.port() {
                        1024 => {}, // Command responce
                        1025 => { // High Priority
                                // first 4 bytes are the sequence number - should check it
                                let mut r = radio_mutex.radio.lock().unwrap();
                                    let previous_ptt = r.ptt;
                                    let previous_dot = r.dot;
                                    let previous_dash = r.dash;
                                    r.ptt = (buffer[4] & 0x01) == 0x01;
                                    r.dot = ((buffer[4] >> 1) & 0x01) == 0x01;
                                    r.dash = ((buffer[4] >> 2) & 0x01) == 0x01;

                                    r.pll_locked = ((buffer[5] >> 2) & 0x01) == 0x01;
                                    r.alex_forward_power = u16::from_be_bytes([buffer[14], buffer[15]]) as i32;
                                    r.alex_reverse_power = u16::from_be_bytes([buffer[22], buffer[23]]) as i32;
                                    r.supply_volts = u16::from_be_bytes([buffer[49], buffer[50]]) as i32;

                                    if r.ptt != previous_ptt {
                                        r.set_state();
                                    }
                                    if r.dot != previous_dot {
                                        r.set_state();
                                    }
                                    if r.dash != previous_dash {
                                        r.set_state();
                                    }

                                drop(r);
                                
                                self.send_high_priority(radio_mutex);
                                },
                        1026 => { // Mic/Line In

                                let data_size = MIC_SAMPLES * MIC_SAMPLE_SIZE;
                                let mut r = radio_mutex.radio.lock().unwrap();
                                if r.transmitter.local_microphone {
                                //if r.audio.local_input {
                                    let mic_buffer = r.audio[0].read_input();
                                    println!("mic_buffer read {}", mic_buffer.len());
                                } else {
                                if !r.transmitter.local_microphone {
                                    let mut sample:i32 = 0;
                                    let mut b = MIC_HEADER_SIZE;
                                    if size >= MIC_HEADER_SIZE + data_size {
                                        for _i in 0..MIC_SAMPLES {
                                            if buffer[b] & 0x80 != 0 {
                                                sample = u32::from_be_bytes([0xFF, 0, buffer[b], buffer[b+1]]) as i32;
                                            } else {
                                                sample = u32::from_be_bytes([0, 0, buffer[b], buffer[b+1]]) as i32;
                                            }
                                            b = b + 2;
                                            let x = microphone_buffer_offset * 2;
                                            if r.tune {
                                                microphone_buffer[x] = 0.0;
                                            } else {
                                                microphone_buffer[x] = sample as f64 / 32768.0;
                                            }
                                            microphone_buffer[x+1] = 0.0;
                                            microphone_buffer_offset = microphone_buffer_offset + 1;
                                            if microphone_buffer_offset >= r.transmitter.microphone_buffer_size {
                                                let raw_ptr: *mut f64 = microphone_buffer.as_mut_ptr() as *mut f64;
                                                let iq_ptr: *mut f64 =  microphone_iq_buffer.as_mut_ptr() as *mut f64;
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
                                                        let ox = tx_iq_buffer_offset * 2;
                                                        tx_iq_buffer[ox] = microphone_iq_buffer[ix] as f64;
                                                        tx_iq_buffer[ox+1] = microphone_iq_buffer[ix+1] as f64;
                                                        tx_iq_buffer_offset = tx_iq_buffer_offset + 1;
                                                        if tx_iq_buffer_offset >= IQ_BUFFER_SIZE {
                                                            self.send_iq_buffer(tx_iq_buffer.clone());
                                                            tx_iq_buffer_offset = 0;
                                                        }
                                                    }
                                                }
                                                microphone_buffer_offset = 0;
                                            }
                                        }
                                    }
                                }
                                }
                                },
                        1027 => {}, // Wide Band IQ samples
                        1035 |
                        1036 |
                        1037 |
                        1038 |
                        1039 |
                        1040 |
                        1041 |
                        1042 => { // RX IQ samples
                            let ddc = (src.port()-1035) as usize;
                            let mut r = radio_mutex.radio.lock().unwrap();
                            let iq_sample_count = u16::from_be_bytes([buffer[14], buffer[15]]) as usize;
                            let data_size = iq_sample_count * SAMPLE_SIZE * INTERLEAVE_FACTOR;
                            let mut i_sample: i32 = 0;
                            let mut q_sample: i32 = 0;
                            let mut b = HEADER_SIZE;
    
                            if size >= HEADER_SIZE + data_size {
                                for _i in 0..iq_sample_count {
                                    if buffer[b] & 0x80 != 0 {
                                        i_sample = u32::from_be_bytes([0xFF, buffer[b], buffer[b+1], buffer[b+2]]) as i32;
                                    } else {
                                        i_sample = u32::from_be_bytes([0, buffer[b], buffer[b+1], buffer[b+2]]) as i32;
                                    }
                                    b = b + 3;
                                    if buffer[b] & 0x80 != 0 {
                                        q_sample = u32::from_be_bytes([0xFF, buffer[b], buffer[b+1], buffer[b+2]]) as i32;
                                    } else {
                                        q_sample = u32::from_be_bytes([0, buffer[b], buffer[b+1], buffer[b+2]]) as i32;
                                    }
                                    b = b + 3;

                                    let i = r.receiver[ddc].samples*2;
                                    r.receiver[ddc].iq_input_buffer[i]=i_sample as f64/16777215.0;
                                    r.receiver[ddc].iq_input_buffer[i+1]=q_sample as f64/16777215.0;
                                    r.receiver[ddc].samples = r.receiver[ddc].samples+1;
                                    if r.receiver[ddc].samples >= r.receiver[ddc].buffer_size {
                                        let raw_ptr: *mut f64 = r.receiver[ddc].iq_input_buffer.as_mut_ptr() as *mut f64;
                                        let audio_ptr: *mut f64 =  audio_buffer.as_mut_ptr() as *mut f64;

                                        if r.receiver[ddc].nb {
                                            unsafe {
                                                xanbEXT(r.receiver[ddc].channel, raw_ptr, raw_ptr);
                                            }
                                        }
                                        if r.receiver[ddc].nb2{
                                            unsafe {
                                                xnobEXT(r.receiver[ddc].channel, raw_ptr, raw_ptr);
                                            }
                                        }

                                        let mut result: c_int = 0;
                                        unsafe {
                                            fexchange0(r.receiver[ddc].channel, raw_ptr, audio_ptr, &mut result);
                                        }
                                        unsafe {
                                            Spectrum0(1, r.receiver[ddc].channel, 0, 0, raw_ptr);
                                        }
                                        r.receiver[ddc].samples = 0;
                                            for i in 0..r.receiver[ddc].output_samples {
                                                let ix = i * 2;
                                                let left_sample: i32 = (audio_buffer[ix] * 32767.0) as i32;
                                                let mut right_sample: i32 = (audio_buffer[ix+1] * 32767.0) as i32;
                                                let rox = r.receiver[ddc].remote_audio_buffer_offset;
                                                if r.audio[ddc].remote_output {
                                                    r.receiver[ddc].remote_audio_buffer[rox] = (left_sample >> 8) as u8;
                                                    r.receiver[ddc].remote_audio_buffer[rox+1] = left_sample as u8;
                                                    r.receiver[ddc].remote_audio_buffer[rox+2] = (right_sample >> 8) as u8;
                                                    r.receiver[ddc].remote_audio_buffer[rox+3] = right_sample as u8;
                                                } else {
                                                    r.receiver[ddc].remote_audio_buffer[rox] = 0;
                                                    r.receiver[ddc].remote_audio_buffer[rox+1] = 0;
                                                    r.receiver[ddc].remote_audio_buffer[rox+2] = 0;
                                                    r.receiver[ddc].remote_audio_buffer[rox+3] = 0;
                                                }
                                                r.receiver[ddc].remote_audio_buffer_offset = r.receiver[ddc].remote_audio_buffer_offset + 4;
                                                if r.receiver[ddc].remote_audio_buffer_offset >= r.receiver[ddc].remote_audio_buffer_size {
                                                    self.send_audio(r.receiver[ddc].clone());
                                                    r.receiver[ddc].remote_audio_buffer_offset = 4;
                                                }

                                                if r.audio[ddc].local_output {
                                                    let lox=r.receiver[ddc].local_audio_buffer_offset * 2;
                                                    if ddc == 0 {
                                                        r.receiver[ddc].local_audio_buffer[lox]=(audio_buffer[ix] * 32767.0) as i16;
                                                        if r.rx2_enabled {
                                                            r.receiver[ddc].local_audio_buffer[lox+1]=0;
                                                        } else {
                                                            r.receiver[ddc].local_audio_buffer[lox+1]=(audio_buffer[ix+1] * 32767.0) as i16;
                                                        }
                                                    } else if ddc == 1 {
                                                        r.receiver[ddc].local_audio_buffer[lox]=0;
                                                        r.receiver[ddc].local_audio_buffer[lox+1]=(audio_buffer[ix+1] * 32767.0) as i16;
                                                    }
                                                    r.receiver[ddc].local_audio_buffer_offset = r.receiver[ddc].local_audio_buffer_offset + 1;
                                                    if r.receiver[ddc].local_audio_buffer_offset == r.receiver[ddc].local_audio_buffer_size {
                                                        r.receiver[ddc].local_audio_buffer_offset = 0;
                                                        let buffer_clone = r.receiver[ddc].local_audio_buffer.clone();
                                                        r.audio[ddc].write_output(&buffer_clone);
                                                    }
                                                }
                                        }
                                    }
                                }
                            }
                        },
                        _ => println!("Unknown port {}", src.port()),
                    }
                }
                Err(e) => {
                    eprintln!("Error receiving UDP packet: {}", e);
                }
            }


            let mut r = radio_mutex.radio.lock().unwrap();
            let updated = r.updated;
            if updated {
                r.updated = false;
            }
            //if r.audio.local_input {
            //    let mic_buffer = r.audio.read_input();
            //    println!("mic_buffer read {}", mic_buffer.len());
            //}
            drop(r);
            if updated {
                self.send_transmit_specific(radio_mutex);
                self.send_receive_specific(radio_mutex);
            }
        }
    }

    pub fn send_general(&mut self) {
        // send to port 1024
        let mut buf = [0u8; 60];
        buf[0] = ((self.general_sequence >> 24) & 0xFF) as u8;
        buf[1] = ((self.general_sequence >> 16) & 0xFF) as u8;
        buf[2] = ((self.general_sequence >> 8) & 0xFF) as u8;
        buf[3] = ((self.general_sequence) & 0xFF) as u8;

        buf[23] = 0x00; // wideband not enabled
        buf[37] = 0x08; // phase word (not frequency)
        buf[38] = 0x01; // enable hardware timer

        buf[58] = 0x01; // enable PA

        if self.device.device == 5 {
          buf[59] = 0x03; // enable ALEX 0 and 1
        } else {
          buf[59] = 0x01; // enable ALEX 0
        }

        self.device.address.set_port(1024);
        self.socket.send_to(&buf, self.device.address).expect("couldn't send data");

        self.general_sequence += 1;
    }

    pub fn send_high_priority(&mut self, radio_mutex: &RadioMutex) {
        // port 1027
        //let r = radio.lock().unwrap();
        let r = radio_mutex.radio.lock().unwrap();
        let tx = &r.transmitter;

        let mut buf = [0u8; 1444];
        buf[0] = ((self.high_priority_sequence >> 24) & 0xFF) as u8;
        buf[1] = ((self.high_priority_sequence >> 16) & 0xFF) as u8;
        buf[2] = ((self.high_priority_sequence >> 8) & 0xFF) as u8;
        buf[3] = ((self.high_priority_sequence) & 0xFF) as u8;
    
        buf[4] = 0x01; // running
        if r.is_transmitting() {
            buf[4] = buf[4] | 0x02;
        }
    
        for i in 0..r.receivers {
            let rx: &Receiver = &r.receiver[i as usize];
            // convert frequency to phase
            let mut f = rx.frequency_a;
            if rx.mode == Modes::CWL.to_usize() {
                 f = f + rx.cw_pitch;
            } else if rx.mode == Modes::CWU.to_usize() {
                 f = f - rx.cw_pitch;
            }

            let mut phase = ((4294967296.0*f)/122880000.0) as u32;
            buf[(9+(i*4)) as usize] = ((phase>>24) & 0xFF) as u8;
            buf[(10+(i*4)) as usize] = ((phase>>16) & 0xFF) as u8;
            buf[(11+(i*4)) as usize] = ((phase>>8) & 0xFF) as u8;
            buf[(12+(i*4)) as usize] = (phase & 0xFF) as u8;

            if r.split {
                f = rx.frequency_b;
                if rx.mode == Modes::CWL.to_usize() {
                     f = f + rx.cw_pitch;
                } else if rx.mode == Modes::CWU.to_usize() {
                     f = f - rx.cw_pitch;
                }
                phase = ((4294967296.0*f)/122880000.0) as u32;
            }
            if i==0 {
                buf[329] = ((phase>>24) & 0xFF) as u8;
                buf[330] = ((phase>>16) & 0xFF) as u8;
                buf[331] = ((phase>>8) & 0xFF) as u8;
                buf[332] = (phase & 0xFF) as u8;
            }
        }



        let mut filter: u32 = 0x00000000;
        if r.is_transmitting() {
            filter |= 0x08000000; // TX_ENABLE
        }

        let mut f = r.receiver[0].frequency_a;
        if f < 1500000.0 {
            filter |= 0x1000;
        } else if f < 2100000.0 {
            filter |= 0x40;
        } else if f < 5500000.0 {
            filter |= 0x20;
        } else if f < 11000000.0 {
            filter |= 0x10;
        } else if f < 22000000.0 {
            filter |= 0x02;
        } else if f < 35000000.0 {
            filter |= 0x04;
        } else {
            filter |= 0x08;
        }


        if self.device.device == 5 { // ORION 2
            if f > 32000000.0 {
                filter |= 0x20000000;
            } else if f > 22000000.0 {
                filter |= 0x40000000;
            } else if f > 11000000.0 {
                filter |= 0x20000000;
            } else if f > 5500000.0 {
                filter |= 0x100000;
            } else if f > 2100000.0 {
                filter |= 0x200000;
            } else if f > 1500000.0 {
                filter |= 0x400000;
            } else {
                filter |= 0x800000;
            }
        } else {
            if f > 35600000.0 {
                filter |= 0x08;
            } else if f > 24000000.0 {
                filter |= 0x04;
            } else if f > 16500000.0 {
                filter |= 0x02;
            } else if f > 8000000.0 {
                filter |= 0x10;
            } else if f > 5000000.0 {
                filter |= 0x20;
            } else if f > 2500000.0 {
                filter |= 0x40;
            } else {
                filter |= 0x40;
            }

        }

        filter |= ALEX_ANTENNA_1;
        
        buf[1432]=((filter >> 24) & 0xFF) as u8;
        buf[1433]=((filter >> 16) & 0xFF) as u8;
        buf[1434]=((filter >> 8) & 0xFF) as u8;
        buf[1435]=(filter & 0xFF) as u8;
 
        buf[1430] = ((filter>>8)&0xFF) as u8;
        buf[1431] = (filter&0xFF) as u8;

        if r.is_transmitting() {
            buf[1443] = 0;
        } else {
            buf[1443] = r.receiver[0].attenuation as u8;
            buf[1442] = r.receiver[1].attenuation as u8;
        }

        self.device.address.set_port(1027);
        self.socket.send_to(&buf, self.device.address).expect("couldn't send data");
        self.high_priority_sequence += 1;
    }

    pub fn send_audio(&mut self, mut rx: Receiver) {
        // port 1028
        rx.remote_audio_buffer[0] = ((self.audio_sequence >> 24) & 0xFF) as u8;
        rx.remote_audio_buffer[1] = ((self.audio_sequence >> 16) & 0xFF) as u8;
        rx.remote_audio_buffer[2] = ((self.audio_sequence >> 8) & 0xFF) as u8;
        rx.remote_audio_buffer[3] = ((self.audio_sequence) & 0xFF) as u8;
        self.device.address.set_port(1028);
        self.socket.send_to(&rx.remote_audio_buffer, self.device.address).expect("couldn't send data");
        self.audio_sequence += 1;
    }

    pub fn send_receive_specific(&mut self, radio_mutex: &RadioMutex) {
        // port 1025
        //let r = radio.lock().unwrap();
        let r = radio_mutex.radio.lock().unwrap();

        let mut buf = [0u8; 1444];
        buf[0] = ((self.receive_specific_sequence >> 24) & 0xFF) as u8;
        buf[1] = ((self.receive_specific_sequence >> 16) & 0xFF) as u8;
        buf[2] = ((self.receive_specific_sequence >> 8) & 0xFF) as u8;
        buf[3] = ((self.receive_specific_sequence) & 0xFF) as u8;

        buf[4] = r.adc.len() as u8;
        for i in 0..r.adc.len() {
            buf[5] |= (r.adc[i].dither as u8) << i;
            buf[6] |= (r.adc[i].random as u8) << i;
        }
        buf[7] = 0x03; // 2 receivers -- DDC0 and DDC1

        for i in 0..r.receivers {
          let mut rx = &r.receiver[i as usize];
          buf[(17+(i*6)) as usize] = 0x00; // ADC to use for DDC0
          buf[(18+(i*6)) as usize] = (((rx.sample_rate/1000)>>8)&0xFF) as u8; // sample_rate
          buf[(19+(i*6)) as usize] = ((rx.sample_rate/1000)&0xFF) as u8; // sample_rate to use for DDC0
          buf[(22+(i*6)) as usize] = 24;  // 24 bits per sample
        }

        self.device.address.set_port(1025);
        self.socket.send_to(&buf, self.device.address).expect("couldn't send data");
        self.receive_specific_sequence += 1;
    }

    pub fn send_transmit_specific(&mut self, radio_mutex: &RadioMutex) {
        // port 1026
        //let r = radio.lock().unwrap();
        let r = radio_mutex.radio.lock().unwrap();
        let tx = &r.transmitter;

        let mut buf = [0u8; 60];
        buf[0] = ((self.transmit_specific_sequence >> 24) & 0xFF) as u8;
        buf[1] = ((self.transmit_specific_sequence >> 16) & 0xFF) as u8;
        buf[2] = ((self.transmit_specific_sequence >> 8) & 0xFF) as u8;
        buf[3] = ((self.transmit_specific_sequence) & 0xFF) as u8;

        buf[4] = 1; // DACs

        buf[5] = 0;
        if tx.mode == Modes::CWL.to_usize()  || tx.mode == Modes::CWU.to_usize() {
            buf[5] |= 0x02;
        }
        if r.cw_keys_reversed {
            buf[5] |= 0x04;
        }     
        if r.cw_keyer_mode == Keyer::ModeA {
            buf[5] |= 0x08;
        }     
        if r.cw_keyer_mode == Keyer::ModeB {
            buf[5] |= 0x28;
        }
        if r.cw_keyer_sidetone_volume != 0 {
            buf[5] |= 0x10;
        } 
        if r.cw_keyer_spacing != 0 {
            buf[5] |= 0x40;
        }
        if r.cw_breakin {
            buf[5] |= 0x80;
        }

        buf[6] = r.cw_keyer_sidetone_volume as u8;
        buf[7] = (r.cw_keyer_sidetone_frequency >> 8) as u8;
        buf[8] = r.cw_keyer_sidetone_frequency as u8;

        buf[9] = r.cw_keyer_speed as u8;
        buf[10] = r.cw_keyer_weight as u8;
        buf[11] = ((r.cw_keyer_hang_time >> 8) & 0xFF) as u8;
        buf[12] = (r.cw_keyer_hang_time &0xFF) as u8;

        buf[50] = 0x00;
        if r.line_in {
            buf[50] |= 0x01;
        }
        if r.mic_boost {
            buf[50] |= 0x02;
        }
        if !r.mic_ptt {
            buf[50] |= 0x04;
        }
        if r.mic_bias_ring { // ptt on tip else bias on tip and ptt on ring
            buf[50] |= 0x08;
        }
        if r.mic_bias_enable {
            buf[50] |= 0x10;
        }
        if r.mic_saturn_xlr {
            buf[50] |= 0x20;
        }

        self.device.address.set_port(1026);
        self.socket.send_to(&buf, self.device.address).expect("couldn't send data");
        self.transmit_specific_sequence += 1;
    }


    fn send_iq_buffer(&mut self, buffer: Vec<f64>) {
        // port 1029
        let mut buf = [0u8; 1444];
        buf[0] = ((self.tx_iq_sequence >> 24) & 0xFF) as u8;
        buf[1] = ((self.tx_iq_sequence >> 16) & 0xFF) as u8;
        buf[2] = ((self.tx_iq_sequence >> 8) & 0xFF) as u8;
        buf[3] = ((self.tx_iq_sequence) & 0xFF) as u8;

        // send 240 24 bit I/Q samples
        let mut b = 4;
        for i in 0..240 {
            let mut ix = i * 2;
            let mut isample = buffer[ix] * 8388607.0;
            if isample>=0.0 {
                isample = (isample + 0.5).floor();
            } else {
                isample = (isample - 0.5).ceil();
            }
            let mut qsample = buffer[ix+1] * 8388607.0;
            if qsample>=0.0 {
                qsample = (qsample + 0.5).floor();
            } else {
                qsample = (qsample - 0.5).ceil();
            }

            buf[b]=(isample as i32 >> 16) as u8;
            buf[b+1]=(isample as i32 >> 8) as u8;
            buf[b+2]=(isample as i32) as u8;
            buf[b+3]=(qsample as i32 >> 16) as u8;
            buf[b+4]=(qsample as i32 >> 8) as u8;
            buf[b+5]=(qsample as i32 ) as u8;

            b = b + 6;
        }

        self.device.address.set_port(1029);
        self.socket.send_to(&buf, self.device.address).expect("couldn't send data");
        self.tx_iq_sequence += 1;
    }

    fn f64_to_f32(input: Vec<f64>) -> Vec<f32> {
        input.into_iter().map(|x| x as f32).collect()
    }

}
