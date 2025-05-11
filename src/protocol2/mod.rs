use nix::sys::socket::setsockopt;
use nix::sys::socket::sockopt::{ReuseAddr, ReusePort};
use rodio::{buffer::SamplesBuffer, OutputStream, Sink, Source};
use std::net::{UdpSocket};
use std::sync::{Arc, Mutex};
use std::os::raw::c_int;

use crate::discovery::Device;
use crate::receiver::Receiver;
use crate::radio::Radio;
use crate::wdsp::*;

const HEADER_SIZE: usize  = 16;  // 16 byte header
const SAMPLE_SIZE: usize = 3;    // 3 byte (24 bit) samples
const INTERLEAVE_FACTOR: usize = 2; // 2 samples (I & Q) interleaved

#[derive(Debug)]
pub struct Protocol2 {
    device: Device, 
    socket: UdpSocket,     
    general_sequence: u32,
    high_priority_sequence: u32,
    receive_specific_sequence: u32,
    transmit_specific_sequence: u32,
    audio_sequence: u32,
}   

impl Protocol2 {

    pub fn new(device: Device, radio: Arc<Mutex<Radio>> ) -> Protocol2 {
        println!("Protocol2::new");
        let socket = UdpSocket::bind("0.0.0.0:0").expect("bind failed");
        setsockopt(&socket, ReusePort, &true).unwrap();
        setsockopt(&socket, ReuseAddr, &true).unwrap();


        let general_sequence: u32 = 0;
        let high_priority_sequence: u32 = 0;
        let receive_specific_sequence: u32 = 0;
        let transmit_specific_sequence: u32 = 0; 
        let audio_sequence: u32 = 0; 

        let p2 = Protocol2{device,
                           socket,
                           general_sequence,
                           high_priority_sequence,
                           receive_specific_sequence,
                           transmit_specific_sequence,
                           audio_sequence};

        p2

    }

    pub fn run(&mut self, device: Device, radio: Arc<Mutex<Radio>>) {
        println!("Protocol2::run");
        let r = radio.lock().unwrap();

        let mut buffer = vec![0; 65536];
        let mut audio_buffer: Vec<f64> = vec![0.0; (r.receiver[0].output_samples*2) as usize];

        println!("Protocol2::setup output stream");
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        sink.play();

        self.send_general();
        self.send_high_priority(r.receiver[0].clone());
        self.send_receive_specific(r.receiver[0].clone());
        drop(r);

        loop {
            match self.socket.recv_from(&mut buffer) {
                Ok((size, src)) => {
                    //println!("recv_from: size={} from {}", size, src.port());
                    match src.port() {
                        1024 => {}, // Command responce
                        1025 => { // High Priority
                                let r = radio.lock().unwrap();
                                self.send_high_priority(r.receiver[0].clone());
                                },
                        1026 => {}, // Mic/Line In
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
                            if ddc == 0 {
                                let mut r = radio.lock().unwrap();
                                let iq_sample_count = u16::from_be_bytes([buffer[14], buffer[15]]) as usize;
                                let data_size = iq_sample_count * SAMPLE_SIZE * INTERLEAVE_FACTOR;
                                let mut i_sample: i32 = 0;
                                let mut q_sample: i32 = 0;
                                let mut b = HEADER_SIZE;
    
                                if size >= HEADER_SIZE + data_size {
                                    for i in 0..iq_sample_count {
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
                                            let mut result: c_int = 0;
                                            unsafe {
                                                fexchange0(r.receiver[ddc].channel, raw_ptr, audio_ptr, &mut result);
                                            }
                                            if result != 0 {
                                                println!("fexchange0: result={}",result);
                                            }
                                            unsafe {
                                                Spectrum0(1, r.receiver[ddc].channel, 0, 0, raw_ptr);
                                            }
                                            r.receiver[ddc].samples = 0;
                                            for i in 0..r.receiver[ddc].output_samples {
                                                let ix = i * 2;
                                                let left_sample: i32 = (audio_buffer[ix] * 32767.0) as i32;
                                                let right_sample: i32 = (audio_buffer[ix+1] * 32767.0) as i32;
                                                let rox = r.receiver[ddc].remote_audio_buffer_offset;
                                                r.receiver[ddc].remote_audio_buffer[rox] = (left_sample >> 8) as u8;
                                                r.receiver[ddc].remote_audio_buffer[rox+1] = left_sample as u8;
                                                r.receiver[ddc].remote_audio_buffer[rox+2] = (right_sample >> 8) as u8;
                                                r.receiver[ddc].remote_audio_buffer[rox+3] = right_sample as u8;
                                                r.receiver[ddc].remote_audio_buffer_offset = r.receiver[ddc].remote_audio_buffer_offset + 4;
                                                if r.receiver[ddc].remote_audio_buffer_offset >= r.receiver[ddc].remote_audio_buffer_size {
                                                    self.send_audio(r.receiver[ddc].clone());
                                                    r.receiver[ddc].remote_audio_buffer_offset = 4;
                                                }

                                                // local audio
                                                let lox=r.receiver[ddc].local_audio_buffer_offset * 2;
                                                r.receiver[ddc].local_audio_buffer[lox]=audio_buffer[ix];
                                                r.receiver[ddc].local_audio_buffer[lox+1]=audio_buffer[ix+1];
                                                r.receiver[ddc].local_audio_buffer_offset = r.receiver[ddc].local_audio_buffer_offset + 1;
                                                if r.receiver[ddc].local_audio_buffer_offset == r.receiver[ddc].local_audio_buffer_size {
                                                    r.receiver[ddc].local_audio_buffer_offset = 0;
                                                    let f32samples = Protocol2::f64_to_f32(r.receiver[ddc].local_audio_buffer.clone());
                                                    let source = SamplesBuffer::new(2, 48000, f32samples);
                                                    if sink.len() < 3 {
                                                        sink.append(source);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            } else {
                                println!("Only RX0 currently supported: this is for rx {}",ddc);
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
        println!("capture_and_process_udp: EXIT");
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
          buf[59] = 0x01; // enable ALEX 0 and 1
        } else {
          buf[59] = 0x01; // enable ALEX 0
        }

        self.device.address.set_port(1024);

        self.socket.send_to(&buf, self.device.address).expect("couldn't send data");
        self.general_sequence += 1;
    }

    pub fn send_high_priority(&mut self, rx: Receiver) {
        // port 1027
        let mut buf = [0u8; 1444];
        buf[0] = ((self.high_priority_sequence >> 24) & 0xFF) as u8;
        buf[1] = ((self.high_priority_sequence >> 16) & 0xFF) as u8;
        buf[2] = ((self.high_priority_sequence >> 8) & 0xFF) as u8;
        buf[3] = ((self.high_priority_sequence) & 0xFF) as u8;
    
        buf[4] = 0x01;
    
        // convert frequency to phase
        let phase = ((4294967296.0*rx.frequency_a)/122880000.0) as u32;
        buf[9] = ((phase>>24) & 0xFF) as u8;
        buf[10] = ((phase>>16) & 0xFF) as u8;
        buf[11] = ((phase>>8) & 0xFF) as u8;
        buf[12] = (phase & 0xFF) as u8;
    
        if self.device.device == 5 {
            buf[13] = ((phase>>24) & 0xFF) as u8;
            buf[14] = ((phase>>16) & 0xFF) as u8;
            buf[15] = ((phase>>8) & 0xFF) as u8;
            buf[16] = (phase & 0xFF) as u8;
        }

        // assume transmit and receive on same frequency
        buf[329] = ((phase>>24) & 0xFF) as u8;
        buf[330] = ((phase>>16) & 0xFF) as u8;
        buf[331] = ((phase>>8) & 0xFF) as u8;
        buf[332] = (phase & 0xFF) as u8;


        buf[345] = 0x80; // half power

        let f = rx.filters;

        buf[1432] = ((f>>24)&0xFF) as u8;
        buf[1433] = ((f>>16)&0xFF) as u8;
        buf[1434] = ((f>>8)&0xFF) as u8;
        buf[1435] = (f&0xFF) as u8;

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

    pub fn send_receive_specific(&mut self, rx: Receiver) {
        // port 1025
        let mut buf = [0u8; 1444];
        buf[0] = ((self.receive_specific_sequence >> 24) & 0xFF) as u8;
        buf[1] = ((self.receive_specific_sequence >> 16) & 0xFF) as u8;
        buf[2] = ((self.receive_specific_sequence >> 8) & 0xFF) as u8;
        buf[3] = ((self.receive_specific_sequence) & 0xFF) as u8;

        if self.device.device == 5 {
            buf[4] = 2; // ADCs
            buf[5] = 0x03;  // enable dither for each ADC
            buf[6] = 0x03;  // enable random for each ADC
            buf[7] = 0x01;  // DDC enable - just 1 receiver
        } else {
            buf[4] = 1; // ADCs
            buf[5] = 0x01;  // enable dither for each ADC
            buf[6] = 0x01;  // enable random for each ADC
            buf[7] = 0x01;  // DDC enable
        }
        buf[17] = 0x00; // ADC to use for DDC0
        buf[18] = (((rx.sample_rate/1000)>>8)&0xFF) as u8; // sample_rate
        buf[19] = ((rx.sample_rate/1000)&0xFF) as u8; // sample_rate to use for DDC0
        buf[22] = 24;  // 24 bits per sample

        if self.device.device == 5 {
            buf[23] = 0x01; // ADC to use for DDC0
            buf[24] = (((rx.sample_rate/1000)>>8)&0xFF) as u8; // sample_rate
            buf[25] = ((rx.sample_rate/1000)&0xFF) as u8; // sample_rate to use for DDC0
            buf[28] = 24;  // 24 bits per sample
        }

        self.device.address.set_port(1025);
        self.socket.send_to(&buf, self.device.address).expect("couldn't send data");
        self.receive_specific_sequence += 1;
    }

    fn f64_to_f32(input: Vec<f64>) -> Vec<f32> {
        input.into_iter().map(|x| x as f32).collect()
    }

}
