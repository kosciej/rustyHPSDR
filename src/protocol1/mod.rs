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
use rodio::{buffer::SamplesBuffer, OutputStream, Sink, Source};
use std::net::{UdpSocket};
use std::sync::{Arc, Mutex};
use std::os::raw::c_int;

use crate::discovery::Device;
use crate::radio::Radio;
use crate::wdsp::*;

const OZY_BUFFER_SIZE: usize = 512;
const METIS_BUFFER_SIZE: usize = (OZY_BUFFER_SIZE * 2) + 8;
const SYNC: u8  = 0x7F;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State{
  SYNC_0=0,
  SYNC_1,
  SYNC_2,
  CONTROL_0,
  CONTROL_1,
  CONTROL_2,
  CONTROL_3,
  CONTROL_4, 
  LEFT_SAMPLE_HI,
  LEFT_SAMPLE_MID,
  LEFT_SAMPLE_LOW,
  RIGHT_SAMPLE_HI,
  RIGHT_SAMPLE_MID,
  RIGHT_SAMPLE_LOW,
  MIC_SAMPLE_HI,
  MIC_SAMPLE_LOW, 
}

pub struct Protocol1 {
    device: Device,
    socket: UdpSocket,
    receive_sequence: u32,
    send_sequence: u32,
    wide_sequence: u32,
    state: State,
    receivers: u8,
    current_receiver: u8,
    c0: u8,
    c1: u8,
    c2: u8,
    c3: u8,
    c4: u8,
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
        let state: State = State::SYNC_0;
        let receivers: u8 = 1;
        let current_receiver: u8 = 0;
        let c0: u8 = 0;
        let c1: u8 = 0;
        let c2: u8 = 0;
        let c3: u8 = 0;
        let c4: u8 = 0;
        let iq_samples: i32 = (512 - 8)/((receivers as i32 * 6) + 2);
        let n_samples: i32 = 0;
        let left_sample: i32 = 0;
        let right_sample: i32 = 0;
        let ozy_buffer: Vec<u8> = vec![0; OZY_BUFFER_SIZE];
        let ozy_buffer_offset: usize = 8;
        let ozy_command: u8 = 1;
        let metis_buffer: Vec<u8> = vec![0; METIS_BUFFER_SIZE];
        let metis_buffer_offset: usize = 8;

        let p1 = Protocol1{device,
                           socket,
                           receive_sequence,
                           send_sequence,
                           wide_sequence,
                           state,
                           receivers,
                           current_receiver,
                           c0,
                           c1,
                           c2,
                           c3,
                           c4,
                           n_samples,
                           iq_samples,
                           left_sample,
                           right_sample,
                           ozy_buffer,
                           ozy_buffer_offset,
                           ozy_command,
                           metis_buffer,
                           metis_buffer_offset,
                          };

        p1
    }

    pub fn run(&mut self, radio: Arc<Mutex<Radio>>) {

        // start the radio running
        let r = radio.lock().unwrap();
        loop {
            if self.device.device == 6 {
                self.send_ozy_buffer(r.receiver[0].frequency_a, r.receiver[0].rxgain, self.device);
            } else {
                self.send_ozy_buffer(r.receiver[0].frequency_a, r.receiver[0].attenuation, self.device);
            }
            if self.ozy_command == 1 {
                break;
            }
        }
        loop {
            if self.device.device == 6 {
                self.send_ozy_buffer(r.receiver[0].frequency_a, r.receiver[0].rxgain, self.device);
            } else {
                self.send_ozy_buffer(r.receiver[0].frequency_a, r.receiver[0].attenuation, self.device);
            }
            if self.ozy_command == 1 {
                break;
            }
        }
        self.metis_start(self.device);
        drop(r);

        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        sink.play();

        let mut buffer = vec![0; 2048];
        loop {
            match self.socket.recv_from(&mut buffer) {
                Ok((_size, src)) => {
                    match src.port() {
                        1024 => {
                                if buffer[0] == 0xEF && buffer[1] == 0xFE {
                                    let seq = u32::from_be_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]);
                                    match buffer[2] {
                                        1 => {
                                             match buffer[3] {
                                                 6 => { // IQ samples
                                                      self.process_ozy_buffer(&buffer,8,radio.clone(), &sink);
                                                      self.process_ozy_buffer(&buffer,520,radio.clone(), &sink);
                                                      },
                                                 4 => { // Wideband samples
                                                      },
                                                 _ => println!("Unexpected EP {}", buffer[3]),
                                             }
                                             },
                                        _ => println!("Unexpected packet type {}", buffer[2]),
                                    }
                                } else {
                                    println!("Received bad header bytes");
                                }
                                },
                        _ => println!("Unknown port {}", src.port()),
                    }
                }
                Err(e) => {
                    eprintln!("Error receiving UDP packet: {}", e);
                }
            }
        }
    }

    fn process_ozy_buffer(&mut self, buffer: &Vec<u8>, offset: usize, radio: Arc<Mutex<Radio>>, sink: &Sink)  {
        let mut r = radio.lock().unwrap();
        let mut audio_buffer: Vec<f64> = vec![0.0; (r.receiver[0].output_samples*2) as usize];
        let mut subrx_audio_buffer: Vec<f64> = vec![0.0; (r.receiver[0].output_samples*2) as usize];
        let mut i_sample = 0;
        let mut q_sample = 0;
        let mut b = offset;
        if buffer[b]==SYNC && buffer[b+1]==SYNC && buffer[b+2]==SYNC {
            b = b + 3;
        } else {
            eprintln!("SYNC error");
            return;
        }
        // collect the control bytes
        b = b + 5;
        for _s in 0..self.iq_samples {
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
            // todo process mic samples
            b = b + 2;

            let i = r.receiver[0].samples*2;
            r.receiver[0].iq_input_buffer[i]=i_sample as f64/16777215.0;
            r.receiver[0].iq_input_buffer[i+1]=q_sample as f64/16777215.0;
            r.receiver[0].samples = r.receiver[0].samples+1;
            if r.receiver[0].samples >= r.receiver[0].buffer_size {
                let raw_ptr: *mut f64 = r.receiver[0].iq_input_buffer.as_mut_ptr() as *mut f64;
                let audio_ptr: *mut f64 =  audio_buffer.as_mut_ptr() as *mut f64;
                let subrx_audio_ptr: *mut f64 =  subrx_audio_buffer.as_mut_ptr() as *mut f64;
                let mut result: c_int = 0;
                unsafe {
                    fexchange0(r.receiver[0].channel, raw_ptr, audio_ptr, &mut result);
                    if r.receiver[0].subrx {
                        fexchange0(r.receiver[0].subrx_channel, raw_ptr, subrx_audio_ptr, &mut result);
                    }
                }
                unsafe {
                    Spectrum0(1, r.receiver[0].channel, 0, 0, raw_ptr);
                }
                r.receiver[0].samples = 0;
                for i in 0..r.receiver[0].output_samples {
                    let ix = i * 2 ;
                    let left_sample: i32 = (audio_buffer[ix] * 16777215.0) as i32;
                    let mut right_sample: i32 = (audio_buffer[ix+1] * 16777215.0) as i32;
                    if r.receiver[0].subrx {
                        right_sample = (subrx_audio_buffer[ix+1] * 16777215.0) as i32;
                    }

                    self.ozy_buffer[self.ozy_buffer_offset] = (left_sample >> 8) as u8;
                    self.ozy_buffer_offset = self.ozy_buffer_offset + 1;
                    self.ozy_buffer[self.ozy_buffer_offset] = left_sample as u8;
                    self.ozy_buffer_offset = self.ozy_buffer_offset + 1;
                    self.ozy_buffer[self.ozy_buffer_offset] = (right_sample >> 8) as u8;
                    self.ozy_buffer_offset = self.ozy_buffer_offset + 1;
                    self.ozy_buffer[self.ozy_buffer_offset] = right_sample as u8;
                    self.ozy_buffer_offset = self.ozy_buffer_offset + 1;
                    self.ozy_buffer[self.ozy_buffer_offset] = 0;
                    self.ozy_buffer_offset = self.ozy_buffer_offset + 1;
                    self.ozy_buffer[self.ozy_buffer_offset] = 0;
                    self.ozy_buffer_offset = self.ozy_buffer_offset + 1;
                    self.ozy_buffer[self.ozy_buffer_offset] = 0;
                    self.ozy_buffer_offset = self.ozy_buffer_offset + 1;
                    self.ozy_buffer[self.ozy_buffer_offset] = 0;
                    self.ozy_buffer_offset = self.ozy_buffer_offset + 1;
                    if self.ozy_buffer_offset == OZY_BUFFER_SIZE {
                        if self.device.device == 6 {
                            self.send_ozy_buffer(r.receiver[0].frequency_a, r.receiver[0].rxgain, self.device);
                        } else {
                            self.send_ozy_buffer(r.receiver[0].frequency_a, r.receiver[0].attenuation, self.device);
                        }
                        self.ozy_buffer_offset = 8;
                    }
                    let ox=r.receiver[0].local_audio_buffer_offset * 2;
                    r.receiver[0].local_audio_buffer[ox+1]=audio_buffer[ix];
                    if r.receiver[0].subrx {
                        r.receiver[0].local_audio_buffer[ox]=subrx_audio_buffer[ix+1];
                    } else {
                        r.receiver[0].local_audio_buffer[ox]=audio_buffer[ix+1];
                    }
                    r.receiver[0].local_audio_buffer_offset = r.receiver[0].local_audio_buffer_offset + 1;
                    if r.receiver[0].local_audio_buffer_offset == r.receiver[0].local_audio_buffer_size {
                        r.receiver[0].local_audio_buffer_offset = 0;
                        let f32samples = Protocol1::f64_to_f32(r.receiver[0].local_audio_buffer.clone());
                        let source = SamplesBuffer::new(2, 48000, f32samples);
                        if sink.len() < 3 {
                           sink.append(source);
                        }
                    }
                }
            }
        }
    }

    fn send_ozy_buffer(&mut self, frequency: f32, attenuation: i32, device: Device) {
                        // send the buffer
                        self.ozy_buffer[0] = SYNC;
                        self.ozy_buffer[1] = SYNC;
                        self.ozy_buffer[2] = SYNC;
                        self.ozy_buffer[3] = 0x00; // C0
                        self.ozy_buffer[4] = 0x00; // C1
                        self.ozy_buffer[5] = 0x00; // C2
                        self.ozy_buffer[6] = 0x00; // C3
                        self.ozy_buffer[7] = 0x00; // C4

                        if self.metis_buffer_offset == 8 {
                            self.ozy_buffer[3] = 0x00; // C0
                            self.ozy_buffer[4] = 0x03; // C1
                            self.ozy_buffer[5] = 0x00; // C2
                            self.ozy_buffer[6] = 0x00; // C3
                            self.ozy_buffer[7] = 0x04; // C4 (1 receiver)
                        } else {
                            match self.ozy_command {
                                1 => {
                                    self.ozy_buffer[3] = 0x02; // C0
                                    // TX frequency
                                    let f: i32 = frequency as i32;
                                    self.ozy_buffer[4] = (f >> 24) as u8; // C1
                                    self.ozy_buffer[5] = (f>>16) as u8; // C2
                                    self.ozy_buffer[6] = (f>>8) as u8; // C3
                                    self.ozy_buffer[7] = f as u8; // C4
                                     },
                                2 => {
                                    self.ozy_buffer[3] = 0x04 + (self.current_receiver * 2); // C0
                                    // RX frequency
                                    let f: i32 = frequency as i32;
                                    self.ozy_buffer[4] = (f >> 24) as u8; // C1
                                    self.ozy_buffer[5] = (f>>16) as u8; // C2
                                    self.ozy_buffer[6] = (f>>8) as u8; // C3
                                    self.ozy_buffer[7] = f as u8; // C4
                                    self.current_receiver = self.current_receiver + 1;
                                    if self.current_receiver >= device.supported_receivers {
                                        self.current_receiver = 0;
                                    }
                                     },
                                3 => {
                                    self.ozy_buffer[3] = 0x12; // C0
                                    self.ozy_buffer[4] = 0x00; // C1
                                    self.ozy_buffer[5] = 0x00; // C2
                                    self.ozy_buffer[6] = 0x00; // C3
                                    self.ozy_buffer[7] = 0x00; // C4
                                     },
                                4 => {
                                    self.ozy_buffer[3] = 0x14; // C0
                                    self.ozy_buffer[4] = 0x01; // C1
                                    self.ozy_buffer[5] = 0x00; // C2
                                    self.ozy_buffer[6] = 0x00; // C3
                                    self.ozy_buffer[7] = 0x00; // C4
                                    if device.device == 6 { // HERMES_LITE
                                        if device.version > 42 { // HERMES_LITE_2
                                            self.ozy_buffer[7] = self.ozy_buffer[7] | 0x40;
                                            self.ozy_buffer[7] = self.ozy_buffer[7] | ((attenuation  + 12) & 0x3F) as u8;
                                        }  else { // HERMES_LITE_1
                                            //self.ozy_buffer[7] = self.ozy_buffer[7] | 0x20;
                                        }
                                    } else {
                                        //if attenuation != 0 {
                                            self.ozy_buffer[7] = 0x20;
                                            self.ozy_buffer[7] = self.ozy_buffer[7] | (attenuation & 0x1F) as u8;
                                        //}
                                    }
                                     },
                                5 => {
                                    self.ozy_buffer[3] = 0x16; // C0
                                    self.ozy_buffer[4] = 0x00; // C1
                                    self.ozy_buffer[5] = 0x00; // C2
                                    self.ozy_buffer[6] = 0x0C; // C3
                                    self.ozy_buffer[7] = 0x1E; // C4
                                     },
                                6 => {
                                    self.ozy_buffer[3] = 0x1C; // C0
                                    self.ozy_buffer[4] = 0x00; // C1
                                    self.ozy_buffer[5] = 0x00; // C2
                                    self.ozy_buffer[6] = 0x00; // C3
                                    self.ozy_buffer[7] = 0x00; // C4
                                     },
                                7 => {
                                    self.ozy_buffer[3] = 0x1E; // C0
                                    self.ozy_buffer[4] = 0x00; // C1
                                    self.ozy_buffer[5] = 0x14; // C2
                                    self.ozy_buffer[6] = 0x14; // C3
                                    self.ozy_buffer[7] = 0x00; // C4
                                     },
                                8 => {
                                    self.ozy_buffer[3] = 0x20; // C0
                                    self.ozy_buffer[4] = 0x00; // C1
                                    self.ozy_buffer[5] = 0x00; // C2
                                    self.ozy_buffer[6] = 0x28; // C3
                                    self.ozy_buffer[7] = 0x0A; // C4
                                     },
                                9 => {
                                    self.ozy_buffer[3] = 0x22; // C0
                                    self.ozy_buffer[4] = 0x19; // C1
                                    self.ozy_buffer[5] = 0x00; // C2
                                    self.ozy_buffer[6] = 0xC8; // C3
                                    self.ozy_buffer[7] = 0x00; // C4
                                     },
                                10 => {
                                    self.ozy_buffer[3] = 0x24; // C0
                                    self.ozy_buffer[4] = 0x00; // C1
                                    self.ozy_buffer[5] = 0x00; // C2
                                    self.ozy_buffer[6] = 0x00; // C3
                                    self.ozy_buffer[7] = 0x00; // C4
                                     },
                                11 => {
                                    self.ozy_buffer[3] = 0x2E; // C0
                                    self.ozy_buffer[4] = 0x00; // C1
                                    self.ozy_buffer[5] = 0x00; // C2
                                    self.ozy_buffer[6] = 0x04; // C3
                                    self.ozy_buffer[7] = 0x15; // C4
                                     },
                                _ => eprintln!("Invalid command {}", self.ozy_command),
                            }
                            if self.current_receiver == 0 {
                                self.ozy_command = self.ozy_command + 1;
                                if self.ozy_command > 11 {
                                    self.ozy_command = 1;
                                }
                            }
                        }

                        self.write_metis(device);

    }

    fn write_metis(&mut self, device: Device) {
        // copy the buffer
        for i in 0..512 {
            self.metis_buffer[i+self.metis_buffer_offset] = self.ozy_buffer[i];
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
            self.socket.send_to(&self.metis_buffer, device.address).expect("couldn't send data");
            self.send_sequence = self.send_sequence + 1;
            self.metis_buffer_offset = 8;
        }
    }

    fn metis_start(&self, device: Device) {
        let mut buf = [0u8; 64];
        buf[0] = 0xEF;
        buf[1] = 0xFE;
        buf[2] = 0x04;
        buf[3] = 0x03;
        self.socket.send_to(&buf, self.device.address).expect("couldn't send data");
    }

    fn metis_stop(&self, device: Device) {
        let mut buf = [0u8; 64];
        buf[0] = 0xEF;
        buf[1] = 0xFE;
        buf[2] = 0x04;
        buf[3] = 0x00;
        self.socket.send_to(&buf, self.device.address).expect("couldn't send data");
    }

    fn f64_to_f32(input: Vec<f64>) -> Vec<f32> {
        input.into_iter().map(|x| x as f32).collect()
    }


}

