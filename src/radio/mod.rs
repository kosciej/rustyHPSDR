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

use gtk::prelude::*;
use gtk::{Label, ToggleButton};
use gtk::cairo::{Context, LineCap, LineJoin}; 
use glib::source::SourceId;
use gdk_pixbuf::Pixbuf;
use gdk_pixbuf::Colorspace;
use pangocairo;

use std::cell::RefCell;
//use std::{env, fs, path::{PathBuf}};
use std::env;
use std::fmt::Write;
use std::fs;
use std::os::raw::c_int;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};


use crate::discovery::Device;
use crate::bands::BandInfo;
use crate::modes::Modes;
use crate::receiver::Receiver;
use crate::transmitter::Transmitter;
use crate::wdsp::*;
use crate::audio::*;
use crate::alex::*;
use crate::adc::*;

#[derive(Serialize, Deserialize)]
enum RadioModels {
    ANAN_10,
    ANAN_10E,
    ANAN_100,
    ANAN_100D,
    ANAN_200D,
    ANAN_7000D,
    ANAN_8000D,
    HERMES,
    HERMES_2,
    ANGELIA,
    ORION,
    ORION_2,
    HERMES_LITE,
    HERMES_LITE_2,
}

#[derive(Serialize, Deserialize)]
enum FilterBoards {
    NONE,
    ALEX,
    APOLLO,
    N2ADR,
}

#[derive(PartialEq, Serialize, Deserialize, Copy, Clone)]
pub enum Keyer {
    Straight,
    ModeA,
    ModeB,
}

impl Keyer {
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(Keyer::Straight),
            1 => Some(Keyer::ModeA),
            2 => Some(Keyer::ModeB),
            _ => None,
        }
    }

    pub fn to_u32(&self) -> u32 {
        *self as u32
    }
}

#[derive(Serialize, Deserialize)]
pub struct Radio {
    pub name: String,
    pub dev: u8,
    pub protocol: u8,
    pub supported_receivers: u8,
    pub active_receiver: usize,
    pub receivers: u8,
    pub rx2_enabled: bool,
    pub split: bool,
    pub receiver: Vec<Receiver>,
    pub band_info: Vec<BandInfo>,
#[serde(skip_serializing, skip_deserializing)]
    pub s_meter_dbm: f64,
#[serde(skip_serializing, skip_deserializing)]
    pub ptt: bool,
#[serde(skip_serializing, skip_deserializing)]
    pub mox: bool,
#[serde(skip_serializing, skip_deserializing)]
    pub vox: bool,
#[serde(skip_serializing, skip_deserializing)]
    pub tune: bool,
#[serde(skip_serializing, skip_deserializing)]
    pub dot: bool,
#[serde(skip_serializing, skip_deserializing)]
    pub dash: bool,
    pub audio: Audio,
    pub transmitter: Transmitter,

    pub filter_board: FilterBoards,
    pub cw_keyer_mode: Keyer,
    pub cw_keyer_internal: bool,
    pub cw_keys_reversed: bool,
    pub cw_keyer_speed: i32,
    pub cw_keyer_weight: i32,
    pub cw_keyer_spacing: i32,
    pub cw_keyer_ptt_delay: i32,
    pub cw_keyer_hang_time: i32,
    pub cw_breakin: bool,
    pub cw_keyer_sidetone_volume: i32,
    pub cw_keyer_sidetone_frequency: i32,

    pub local_microphone: bool,
    pub local_microphone_name: String,

    pub adc: Vec<Adc>,

    pub alex: u32,

#[serde(skip_serializing, skip_deserializing)]
    pub updated: bool,

#[serde(skip_serializing, skip_deserializing)]
    pub pll_locked: bool,
#[serde(skip_serializing, skip_deserializing)]
    pub adc_overload: bool,
#[serde(skip_serializing, skip_deserializing)]
    pub exciter_power: i32,
#[serde(skip_serializing, skip_deserializing)]
    pub alex_forward_power: i32,
#[serde(skip_serializing, skip_deserializing)]
    pub alex_reverse_power: i32,
#[serde(skip_serializing, skip_deserializing)]
    pub supply_volts: i32,

#[serde(skip_serializing, skip_deserializing)]
    pub swr: f32,

    pub line_in: bool,
    pub mic_boost: bool,
    pub mic_ptt: bool,
    pub mic_bias_ring: bool,
    pub mic_bias_enable: bool,
    pub mic_saturn_xlr: bool,
   
    pub waterfall_auto: bool,
    pub waterfall_calibrate: f32,

#[serde(skip_serializing, skip_deserializing)]
    pub spectrum_timeout_id: Option<SourceId>,
#[serde(skip_serializing, skip_deserializing)]
    pub waterfall_timeout_id: Option<SourceId>,
#[serde(skip_serializing, skip_deserializing)]
    pub meter_timeout_id: Option<SourceId>,
}

#[derive(Clone)]
pub struct RadioMutex {
    pub radio: Arc<Mutex<Radio>>,
}

impl RadioMutex {
    pub fn new(radio: Arc<Mutex<Radio>>) -> Self {
        RadioMutex {
            radio,
        }
    }
 
    pub fn update_spectrum(&self, width: i32) -> (c_int, Vec<f32>) {
        let mut r = self.radio.lock().unwrap();
        let (flag, pixels) = r.update_spectrum(width);
        (flag, pixels)
    }

    pub fn update_spectrum_2(&self, width: i32) -> (c_int, Vec<f32>) {
        let mut r = self.radio.lock().unwrap();
        let (flag, pixels) = r.update_spectrum_2(width);
        (flag, pixels)
    }

    pub fn update_waterfall(&self, width: i32) -> (c_int, Vec<f32>) {
        let mut r = self.radio.lock().unwrap();
        let (flag, pixels) = r.update_waterfall(width);
        (flag, pixels)
    }

    pub fn update_waterfall_2(&self, width: i32) -> (c_int, Vec<f32>) {
        let mut r = self.radio.lock().unwrap();
        let (flag, pixels) = r.update_waterfall_2(width);
        (flag, pixels)
    }

}

impl Radio {

    pub fn new(device: Device, spectrum_width: i32) -> Radio {
        let name = "HPSDR".to_string();
        let dev = device.device;
        let protocol = device.protocol;
        let supported_receivers = device.supported_receivers;
        let active_receiver = 0;
        let receivers: u8 = 2;
        let rx2_enabled: bool = true;
        let split: bool = false;
        let mut receiver: Vec<Receiver> = Vec::new();
        let band_info = BandInfo::new();
        for i in 0..receivers {
            receiver.push(Receiver::new(i, device.protocol, spectrum_width));
        }
        let s_meter_dbm = -121.0;
        let ptt = false;
        let mox = false;
        let vox = false;
        let tune = false;
        let dot = false;
        let dash = false;
        let audio = Audio::new();
        let transmitter = Transmitter::new(8, device.protocol);

        let filter_board = FilterBoards::ALEX;
        let cw_keyer_mode = Keyer::Straight;
        let cw_keyer_internal = true;
        let cw_keys_reversed = false;
        let cw_keyer_speed = 12;
        let cw_keyer_weight = 30;
        let cw_keyer_spacing = 0;
        let cw_keyer_ptt_delay = 20;
        let cw_keyer_hang_time = 300;
        let cw_breakin = false;
        let cw_keyer_sidetone_volume = 20;
        let cw_keyer_sidetone_frequency = 650;
        let local_microphone = false;
        let local_microphone_name = "".to_string();

        let mut adc: Vec<Adc> = Vec::new();
        for i in 0..device.adcs {
            adc.push(Adc::new());
        }
        let alex = ALEX_ANTENNA_1;

        let updated = false;

        let pll_locked = false;
        let adc_overload = false;
        let exciter_power = 0;
        let alex_forward_power = 0;
        let alex_reverse_power = 0;
        let supply_volts = 0;
        let swr = 1.0;

        let line_in = false;
        let mic_boost = true;
        let mic_ptt = true;
        let mic_bias_ring = false;
        let mic_bias_enable = true;
        let mic_saturn_xlr = false;

        let waterfall_auto = true;
        let waterfall_calibrate = 2.0;

        let spectrum_timeout_id = None;
        let waterfall_timeout_id = None;
        let meter_timeout_id = None;

        Radio {
            name,
            dev,
            protocol,
            supported_receivers,
            active_receiver,
            receivers,
            rx2_enabled,
            split,
            receiver,
            band_info,
            s_meter_dbm,
            ptt,
            mox,
            vox,
            tune,
            dot,
            dash,
            audio,
            transmitter,
            filter_board,
            cw_keyer_mode,
            cw_keyer_internal,
            cw_keys_reversed,
            cw_keyer_speed,
            cw_keyer_weight,
            cw_keyer_spacing,
            cw_keyer_ptt_delay,
            cw_keyer_hang_time,
            cw_breakin,
            cw_keyer_sidetone_volume,
            cw_keyer_sidetone_frequency,

            local_microphone,
            local_microphone_name,

            adc,
            alex,

            updated,

            pll_locked,
            adc_overload,
            exciter_power,
            alex_forward_power,
            alex_reverse_power,
            supply_volts,

            swr,
            line_in,
            mic_boost,
            mic_ptt,
            mic_bias_ring,
            mic_bias_enable,
            mic_saturn_xlr,
            waterfall_auto,
            waterfall_calibrate,

            spectrum_timeout_id,
            waterfall_timeout_id,
            meter_timeout_id,
        }
    }

    pub fn init(&mut self) {
        self.s_meter_dbm = -121.0;
        self.ptt = false;
        self.mox = false;
        self.vox = false;
        self.tune = false;
        self.dot = false;
        self.dash = false;
        self.updated = false;

        self.pll_locked = false;
        self.adc_overload = false;
        self.exciter_power = 0;
        self.alex_forward_power = 0;
        self.alex_reverse_power = 0;
        self.supply_volts = 0;

        self.swr = 1.0;
    }

    pub fn is_transmitting(&self) -> bool {
        self.mox | self.ptt | self.vox | self.tune
    }

    pub fn run(&self) {
    }

    pub fn update_spectrum(&mut self, width: i32) -> (c_int, Vec<f32>) {
        let mut zoom = self.receiver[0].zoom;
        let mut channel = self.receiver[0].channel;
        if self.is_transmitting() {
            zoom = 1;
            channel = self.transmitter.channel;
        }
        let mut pixels_len = width * zoom;
        if self.is_transmitting() {
            pixels_len = width * 12;
        } 
        let mut pixels = vec![0.0; pixels_len as usize];
        let mut flag: c_int = 0;
        if pixels.len() != 0 { // may happen at start of application before spectrum is setup
            unsafe {
                GetPixels(channel, 0, pixels.as_mut_ptr(), &mut flag);
            }
        }
        (flag, pixels)
    }
    
    pub fn update_spectrum_2(&mut self, width: i32) -> (c_int, Vec<f32>) {
        let mut zoom = self.receiver[1].zoom;
        let mut channel = self.receiver[1].channel;
        //if self.is_transmitting() {
        //    zoom = 1;
        //    channel = self.transmitter.channel;
        //}
        let mut pixels_len = width * zoom;
        //if self.is_transmitting() {
        //    pixels_len = width * 12;
        //} 
        let mut pixels = vec![0.0; pixels_len as usize];
        let mut flag: c_int = 0;
        if pixels.len() != 0 { // may happen at start of application before spectrum is setup
            unsafe {
                GetPixels(channel, 0, pixels.as_mut_ptr(), &mut flag);
            }
        }
        (flag, pixels)
    }
    
    pub fn update_waterfall(&mut self, width: i32) -> (c_int, Vec<f32>) {
        let mut zoom = self.receiver[self.active_receiver].zoom;
        let mut channel = self.receiver[self.active_receiver].channel;
        if self.is_transmitting() {
            zoom = 1;
            channel = self.transmitter.channel;
        }

        let mut pixels_len = width * zoom;
        if self.is_transmitting() {
            pixels_len = width * 12;
        }

        let mut pixels = vec![0.0; pixels_len as usize];
        let mut flag: c_int = 0;
        if pixels.len() != 0 { // may happen at start of application before spectrum is setup
            unsafe {
                GetPixels(channel, 1, pixels.as_mut_ptr(), &mut flag);
            }
        }
        (flag, pixels)
    }

    pub fn update_waterfall_2(&mut self, width: i32) -> (c_int, Vec<f32>) {
        let mut zoom = self.receiver[1].zoom;
        let mut channel = self.receiver[1].channel;
        if self.is_transmitting() {
            zoom = 1;
            channel = self.transmitter.channel;
        }

        let mut pixels_len = width * zoom;
        if self.is_transmitting() {
            pixels_len = width * 12;
        }

        let mut pixels = vec![0.0; pixels_len as usize];
        let mut flag: c_int = 0;
        if pixels.len() != 0 { // may happen at start of application before spectrum is setup
            unsafe {
                GetPixels(channel, 1, pixels.as_mut_ptr(), &mut flag);
            }
        }
        (flag, pixels)
    }

    pub fn set_state(&self) {
        if self.is_transmitting() {
            unsafe {
                SetChannelState(self.receiver[self.active_receiver].channel, 0, 1);
                SetChannelState(self.transmitter.channel, 1, 0);
            }
        } else {
            unsafe {
                SetChannelState(self.transmitter.channel, 0, 0);
                SetChannelState(self.receiver[self.active_receiver].channel, 1, 0);
            }
        }
    }

    fn config_file_path(device: Device) -> PathBuf {
        let d = format!("{:02X}-{:02X}-{:02X}-{:02X}-{:02X}-{:02X}", device.mac[0], device.mac[1], device.mac[2], device.mac[3], device.mac[4], device.mac[5]);
        let app_name = env!("CARGO_PKG_NAME");
        let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        config_dir.join(app_name).join(d).join("radio.ron")
    }

    pub fn load(device: Device, spectrum_width: i32) -> Self {
        let path = Self::config_file_path(device);
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(s) => match ron::from_str::<Radio>(&s) {
                    Ok(mut radio) => {
                        radio.init();
                        radio
                    }
                    Err(_e) => {
                        Self::new(device, spectrum_width)
                    }
                },
                Err(_e) => {
                    Self::new(device, spectrum_width)
                }
            }
        } else {
            Self::new(device, spectrum_width)
        }
    }

    pub fn save(&self, device: Device) {
        let path = Self::config_file_path(device);
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                if let Err(_e) = fs::create_dir_all(parent) {
                    return;
                }
            }
        }

        match ron::to_string(self) {
            Ok(s) => {
                if let Err(e) = fs::write(&path, s) {
                    eprintln!("Error writing config file {:?}: {}", path, e);
                } else {
                    println!("Successfully saved data to {:?}", path);
                }
            }
            Err(e) => {
                eprintln!("Error serializing data: {}", e);
            }
        }
    }

pub fn draw_spectrum(radio_mutex: &RadioMutex, cr: &Context, width: i32, height: i32, pixels: &Vec<f32>) {
    cr.set_source_rgb(0.0, 0.0, 0.0);
    cr.paint().unwrap();

    let r = radio_mutex.radio.lock().unwrap();   
    let rx = r.active_receiver;

    if r.is_transmitting() {
        // draw the spectrum
        let dbm_per_line: f32 = height as f32/(r.transmitter.spectrum_high-r.transmitter.spectrum_low);
        let spectrum_high = r.transmitter.spectrum_high;
        let spectrum_width = r.transmitter.spectrum_width;

        let mut multiplier = 3; // protocol 1
        if r.protocol == 2 {
            multiplier = 12; // protocol 2
        }
        let pixel_len = spectrum_width * multiplier;

        let hz_per_pixel = r.transmitter.output_rate as f32 / pixel_len as f32;

        if pixels.len() == pixel_len as usize {
            cr.set_source_rgb(1.0, 1.0, 0.0);
            cr.move_to(0.0, height as f64);
            let offset=(pixel_len as f32 / 2.0)-(spectrum_width as f32 / 2.0);
            for i in 0..spectrum_width {
                let pixel = pixels[(i + offset as i32) as usize];
                let y = ((spectrum_high - pixel as f32) * dbm_per_line).floor();
                cr.line_to(i as f64, y.into());
            }
            cr.line_to(spectrum_width as f64, height as f64);
            cr.stroke().unwrap();
        }

        // draw the filter
        cr.set_source_rgba (0.5, 0.5, 0.5, 0.50);
        let center = spectrum_width / 2;
        let filter_left = center as f32 + (r.transmitter.filter_low as f32 / hz_per_pixel);
        let filter_right = center as f32 + (r.transmitter.filter_high as f32 / hz_per_pixel);
        cr.rectangle(filter_left.into(), 0.0, (filter_right-filter_left).into(), height.into());
        let _ = cr.fill();

        // draw the cursor
        cr.set_source_rgb (1.0, 0.0, 0.0);
        cr.set_line_width(1.0);
        cr.move_to((width/2).into(), 0.0);
        cr.line_to((width/2).into(), height.into());
        cr.stroke().unwrap(); 

    } else {
            let b = r.receiver[rx].band.to_usize();
            let dbm_per_line: f32 = height as f32/(r.band_info[b].spectrum_high-r.band_info[b].spectrum_low);

            cr.set_source_rgb(1.0, 1.0, 0.0);                   
            cr.set_line_width(1.0);
            cr.set_line_cap(LineCap::Round);
            cr.set_line_join(LineJoin::Round);

            let frequency_low = r.receiver[rx].frequency_a - (r.receiver[rx].sample_rate/2) as f32;
            let frequency_high = r.receiver[rx].frequency_a + (r.receiver[rx].sample_rate/2) as f32;
            let frequency_range = frequency_high - frequency_low;
            //let hz_per_pixel = frequency_range as f32 / pixels.len() as f32;
    
            let display_frequency_range = frequency_range / r.receiver[rx].zoom as f32;
            let display_frequency_offset = ((frequency_range - display_frequency_range) / 100.0) * r.receiver[rx].pan as f32;
            let display_frequency_low = frequency_low + display_frequency_offset;
            let display_frequency_high = frequency_high + display_frequency_offset;
            let display_hz_per_pixel = display_frequency_range as f32 / width as f32;

            cr.set_source_rgb(0.5, 0.5, 0.5);
            let mut step = 25000.0;  
            match r.receiver[rx].sample_rate {       
                 48000 => step = 50000.0,
                 96000 => step = 10000.0,   
                192000 => step = 20000.0,
                384000 => match r.receiver[rx].zoom {
                              1 => step = 25000.0,
                              2 => step = 25000.0,
                              3 => step = 10000.0,
                              4 => step = 10000.0,
                              5 => step = 10000.0,
                              6 => step = 5000.0,
                              7 => step = 5000.0,
                              8 => step = 5000.0,
                              9 => step = 5000.0,
                              10 => step = 5000.0,
                              11 => step = 2000.0,
                              12 => step = 2000.0,
                              13 => step = 2000.0,
                              14 => step = 2000.0,
                              15 => step = 2000.0,
                              16 => step = 2000.0,
                              _ => step = 25000.0,
                          },
                768000 => step = 50000.0,
               1536000 => step = 100000.0,
                     _ => step = 25000.0,
            }
       
            // draw the frequency markers
            let mut f: f32 = (((display_frequency_low as i32 + step as i32) / step as i32) * step as i32) as f32;
            while f < display_frequency_high {
                let x = (f - display_frequency_low) / display_hz_per_pixel;
                cr.set_source_rgb(0.5, 0.5, 0.5);
                cr.move_to( x.into(), 0.0);
                cr.line_to( x.into(), height.into());
                cr.stroke().unwrap();
                let text = format_u32_with_separators((f / 1000.0) as u32);
                cr.set_source_rgb(1.0, 1.0, 0.0);
                let pango_layout = pangocairo::functions::create_layout(cr);
                pango_layout.set_text(&text);
                let (text_width, _text_height) = pango_layout.pixel_size();
                cr.move_to( (x - (text_width as f32 / 2.0)).into(), 20.0);
                let _ = cr.show_text(&text);
                f = f + step as f32;
            }

            // draw the band limits
            cr.set_source_rgb(1.0, 0.0, 0.0);
            if display_frequency_low < r.band_info[b].low && display_frequency_high > r.band_info[b].low {
                let x = (r.band_info[b].low - display_frequency_low) / display_hz_per_pixel;
                cr.move_to( x.into(), 0.0);
                cr.line_to( x.into(), height.into());
            }

            if display_frequency_low < r.band_info[b].high && display_frequency_high > r.band_info[b].high {
                let x = (r.band_info[b].high - display_frequency_low) / display_hz_per_pixel;
                cr.move_to( x.into(), 0.0);
                cr.line_to( x.into(), height.into());
            }
            cr.stroke().unwrap();


            // draw signal levels
            for i in r.band_info[b].spectrum_low as i32 .. r.band_info[b].spectrum_high as i32 {
                if i % r.receiver[rx].spectrum_step as i32 == 0 {
                    let y = (r.band_info[b].spectrum_high - i as f32) * dbm_per_line;
                    cr.set_source_rgb(0.5, 0.5, 0.5);
                    cr.move_to(0.0, y.into());
                    cr.line_to(width as f64, y.into());
                    cr.stroke().unwrap();
                    let text = format!("{} dBm", i);
                    cr.set_source_rgb(1.0, 1.0, 0.0);
                    cr.move_to( 5.0, (y-2.0).into());
                    let _ = cr.show_text(&text);
                }
            }

            // draw the spectrum
            let spectrum_high = r.band_info[b].spectrum_high;
            let spectrum_width = r.receiver[rx].spectrum_width;
            let pan = ((pixels.len() as f32 - spectrum_width as f32) / 100.0) * r.receiver[rx].pan as f32;
            cr.set_source_rgb(1.0, 1.0, 0.0);
            cr.move_to(0.0, height as f64);
            for i in 0..spectrum_width {
                let pixel = pixels[i as usize + pan as usize];
                let y = ((spectrum_high - pixel as f32) * dbm_per_line).floor();  
                cr.line_to(i as f64, y.into());
            }
            cr.line_to(width as f64, height as f64);

/*
            // fill the spectrum
            let pattern = LinearGradient::new(0.0, (height-20) as f64, 0.0, 0.0);
            let mut s9: f32 = -73.0;
            s9 = ((r.band_info[b].spectrum_high - s9)
                          * (height-20) as f32
                        / (r.band_info[b].spectrum_high - r.band_info[b].spectrum_low)).floor();
            s9 = 1.0-(s9/(height-20) as f32);
            pattern.add_color_stop_rgb(0.0,0.0,1.0,0.0); // Green
            pattern.add_color_stop_rgb((s9/3.0).into(),1.0,0.65,0.0); // Orange
            pattern.add_color_stop_rgb(((s9/3.0)*2.0).into(),1.0,1.0,0.0); // Yellow
            pattern.add_color_stop_rgb(s9.into(),1.0,0.0,0.0); // Red
            cr.set_source(&pattern).expect("Failed to set source");
            cr.close_path();
            let _ = cr.fill_preserve();
*/
            cr.stroke().unwrap();

            let mut frequency = r.receiver[rx].frequency_a;
            if r.receiver[rx].ctun {
                frequency = r.receiver[rx].ctun_frequency;
            }
            if r.receiver[rx].mode == Modes::CWL.to_usize() {
                frequency = frequency + r.receiver[rx].cw_pitch;
            } else if r.receiver[rx].mode == Modes::CWU.to_usize() {
                frequency = frequency - r.receiver[rx].cw_pitch;
            }

         
            // see if cursor and filter visible
            if display_frequency_low < frequency && display_frequency_high > frequency {
                // draw the center line frequency marker
                let x = (frequency - display_frequency_low) / display_hz_per_pixel;
                cr.set_source_rgb(1.0, 0.0, 0.0);
                cr.set_line_width(1.0);
                cr.move_to(x.into(), 0.0);
                cr.line_to(x.into(), height.into());
                cr.stroke().unwrap();
            
                // draw the filter
                cr.set_source_rgba (0.5, 0.5, 0.5, 0.50);
                let filter_left = ((frequency + r.receiver[rx].filter_low) - display_frequency_low) / display_hz_per_pixel;
                let filter_right = ((frequency + r.receiver[rx].filter_high) - display_frequency_low) / display_hz_per_pixel;
                cr.rectangle(filter_left.into(), 0.0, (filter_right-filter_left).into(), height.into());
                let _ = cr.fill();
            }

        }
    }
}

fn format_u32_with_separators(value: u32) -> String {
    let mut result = String::new();
    let value_str = value.to_string(); 
    let len = value_str.len();

    // Iterate over the characters and insert separators
    for (i, ch) in value_str.chars().enumerate() {
        if (len - i) % 3 == 0 && i != 0 {
            write!(&mut result, ".").unwrap();
        }
        write!(&mut result, "{}", ch).unwrap();
    }                                   

    result
}

fn spectrum_waterfall_clicked(radio: &Arc<Mutex<Radio>>, fa: &Label, fb: &Label, x: f64, width: i32, button: u32) {
    let mut r = radio.lock().unwrap();
    let rx = r.active_receiver;
        
    let frequency_low = r.receiver[rx].frequency_a - (r.receiver[rx].sample_rate/2) as f32;
    let frequency_high = r.receiver[rx].frequency_a + (r.receiver[rx].sample_rate/2) as f32;
    let frequency_range = frequency_high - frequency_low;
                
    let display_frequency_range = frequency_range / r.receiver[rx].zoom as f32;
    let display_frequency_offset = ((frequency_range - display_frequency_range) / 100.0) * r.receiver[rx].pan as f32;  
    let display_frequency_low = frequency_low + display_frequency_offset;
    let display_hz_per_pixel = display_frequency_range as f32 / width as f32;


    let f1 = display_frequency_low + (x as f32 * display_hz_per_pixel);
    let f1 = (f1 as u32 / r.receiver[rx].step as u32 * r.receiver[rx].step as u32) as f32;
 
    if r.receiver[rx].ctun {
        r.receiver[rx].ctun_frequency = f1;
        r.receiver[rx].set_ctun_frequency();
        let formatted_value = format_u32_with_separators(r.receiver[rx].ctun_frequency as u32);
        fa.set_label(&formatted_value);
    } else {
        r.receiver[rx].frequency_a = f1;
        let formatted_value = format_u32_with_separators(r.receiver[rx].frequency_a as u32);
        fa.set_label(&formatted_value);
    }
}

fn spectrum_waterfall_scroll(radio: &Arc<Mutex<Radio>>, f: &Label, dy: f64) {
    let mut r = radio.lock().unwrap();
    let rx = r.active_receiver;
    let frequency_low = r.receiver[rx].frequency_a - (r.receiver[rx].sample_rate/2) as f32;
    let frequency_high = r.receiver[rx].frequency_a + (r.receiver[rx].sample_rate/2) as f32;
    if r.receiver[rx].ctun {
        r.receiver[rx].ctun_frequency = r.receiver[rx].ctun_frequency - (r.receiver[rx].step * dy as f32);
        if r.receiver[rx].ctun_frequency < frequency_low {
            r.receiver[rx].ctun_frequency = frequency_low;
        } else if r.receiver[rx].ctun_frequency > frequency_high {
            r.receiver[rx].ctun_frequency = frequency_high;
        }
        let formatted_value = format_u32_with_separators(r.receiver[rx].ctun_frequency as u32);
        f.set_label(&formatted_value);
        r.receiver[rx].set_ctun_frequency();
    } else {
        r.receiver[rx].frequency_a = r.receiver[rx].frequency_a - (r.receiver[rx].step * dy as f32);
        let formatted_value = format_u32_with_separators(r.receiver[rx].frequency_a as u32);
        f.set_label(&formatted_value);
    }
}

fn draw_meter(cr: &Context, dbm: f64) {
    let x_offset = 5.0;
    let mut y_offset = 0.0;
    let db = 1.0; // size in pixels of each dbm

    cr.set_source_rgb(0.0, 1.0, 0.0);
    cr.rectangle(x_offset, 0.0+y_offset, (dbm + 121.0) + db, 10.0);
    let _ = cr.fill();

    cr.set_source_rgb (0.0, 0.0, 0.0);
    for i in 0..54 {
        cr.move_to(x_offset+(i as f64 * db),10.0+y_offset);
        if i%18 == 0 {
            cr.line_to(x_offset+(i as f64 * db),0.0+y_offset);
        } else if i%6 == 0 {
            cr.line_to(x_offset+(i as f64 * db),5.0+y_offset);
        }
    }
    cr.move_to(x_offset+(54.0*db),10.0+y_offset);
    cr.line_to(x_offset+(54.0*db),0.0+y_offset);
    cr.move_to(x_offset+(74.0*db),10.0+y_offset);
    cr.line_to(x_offset+(74.0*db),0.0+y_offset);
    cr.move_to(x_offset+(94.0*db),10.0+y_offset);
    cr.line_to(x_offset+(94.0*db),0.0+y_offset);
    cr.move_to(x_offset+(114.0*db),10.0+y_offset);
    cr.line_to(x_offset+(114.0*db),0.0+y_offset);
    cr.stroke().unwrap();

    cr.move_to(x_offset+(18.0*db)-3.0,20.0+y_offset);
    let _ = cr.show_text("3");
    cr.move_to(x_offset+(36.0*db)-3.0,20.0+y_offset);
    let _ = cr.show_text("6");
    cr.move_to(x_offset+(54.0*db)-3.0,20.0+y_offset);
    let _ = cr.show_text("9");
    cr.move_to(x_offset+(74.0*db)-9.0,20.0+y_offset);
    let _ = cr.show_text("+20");
    cr.move_to(x_offset+(94.0*db)-9.0,20.0+y_offset);
    let _ = cr.show_text("+40");
    cr.move_to(x_offset+(114.0*db)-9.0,20.0+y_offset);
    let _ = cr.show_text("+60");

}


fn draw_waterfall(cr: &Context, width: i32, height: i32, pixbuf: &Rc<RefCell<Option<Pixbuf>>>) {
    let pixbuf_ref = pixbuf.borrow();
    if let Some(pixbuf) = pixbuf_ref.as_ref() {
        cr.set_source_pixbuf(pixbuf, 0.0, 0.0);
        cr.paint().unwrap();
    } else {
        cr.set_source_rgb(0.0, 0.0, 0.0);
        cr.rectangle(0.0, 0.0, width as f64, height as f64);
        cr.fill().unwrap();
    }
}

fn update_waterfall(width: i32, height: i32, radio: &Arc<Mutex<Radio>>, pixbuf: &RefCell<Option<Pixbuf>>, new_pixels: &Vec<f32>) {
    let mut r = radio.lock().unwrap();
    let rx = r.active_receiver;
    let mut average = 0.0;
    let new_pixbuf = match pixbuf.borrow_mut().as_ref() {
        Some(old_pixbuf) => {
            let new_pixbuf = old_pixbuf.clone();
            unsafe {
                let pixels = new_pixbuf.pixels();
                let row_size = width * 3;

                // copy the current waterfall down one line
                for y in (0..height - 1).rev() { // Iterate in reverse order
                    let src_offset = (y * row_size) as usize;
                    let dest_offset = ((y + 1) * row_size) as usize;
                    pixels.copy_within(src_offset..src_offset + row_size as usize, dest_offset);
                }

                // fill in the top line with the latest spectrum data
                let spectrum_width = r.receiver[rx].spectrum_width;
                let pan = ((new_pixels.len() as f32 - spectrum_width as f32) / 100.0) * r.receiver[rx].pan as f32;

                let b = r.receiver[rx].band.to_usize();
                for x in 0..spectrum_width {
                    let mut value: f32 = new_pixels[x as usize + pan as usize] as f32;
                    average += value;
                    if value < r.band_info[b].waterfall_low {
                        value = r.band_info[b].waterfall_low;
                    } else if value > r.band_info[b].waterfall_high {
                        value = r.band_info[b].waterfall_high;
                    }
                    let percent = 100.0 / ((r.band_info[b].waterfall_high - r.band_info[b].waterfall_low) / (value-r.band_info[b].waterfall_low));
                    let mut r = 0.0;
                    let mut g = 0.0;
                    let mut b = 0.0;
                    if percent < 5.0 { r = 0.0; g = 0.0; b = 0.0;
                    } else if percent < 20.0 { r = 255.0; g = 255.0; b = 0.0;
                    } else if value < 50.0 { r = 255.0; g = 125.0; b = 0.0;
                    } else { r = 255.0; g = 0.0; b = 0.0; }

                    let ix = (x * 3) as usize;
                    pixels[ix] = r as u8;
                    pixels[ix + 1] = g as u8;
                    pixels[ix + 2] = b as u8;
                }
                if r.waterfall_auto {
                    r.band_info[b].waterfall_low = (average / spectrum_width as f32) + r.waterfall_calibrate; 
                }
            } // unsafe
            new_pixbuf
        }
        None => {
            let new_pixbuf = Pixbuf::new(Colorspace::Rgb, false, 8, width, height).unwrap();
            unsafe {
                let pixels = new_pixbuf.pixels();
                for x in 0..width {
                    let value = 0.0;
                    let color = (value * 255.0) as u8;
                    let offset = (((height - 1) * width + x) * 3) as usize;
                    pixels[offset] = color;
                    pixels[offset + 1] = color;
                    pixels[offset + 2] = color;
                }
            }
            new_pixbuf
        }
    };

    *pixbuf.borrow_mut() = Some(new_pixbuf);
}
