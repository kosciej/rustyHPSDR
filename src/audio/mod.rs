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

use alsa::pcm::*;
use alsa::device_name::HintIter;
use alsa::{Direction, ValueOr, Error};
use gtk::prelude::*;
use gtk::{Align, CheckButton, ComboBoxText, Grid, Label};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::SystemTime;

use crate::receiver::Receiver;

#[derive(Default, Deserialize, Serialize)]
pub struct Audio {
    pub remote_input: bool,
    pub local_input: bool,
    pub input_device: String,
    #[serde(skip_serializing, skip_deserializing)]
    input: Option<PCM>,
    pub remote_output: bool,
    pub local_output: bool,
    pub output_device: String,
    #[serde(skip_serializing, skip_deserializing)]
    output: Option<PCM>,
}

impl Audio {

    pub fn new() -> Audio {
        let remote_input = true;
        let local_input = false;
        let input_device = String::from("default"); 
        let input = None;
        let remote_output = true;
        let local_output = false;
        let output_device = String::from("default"); 
        let output = None;

        let audio = Audio{remote_input, local_input, input_device, input, remote_output, local_output, output_device, output};
        audio
    }

    pub fn init(&mut self) {
        self.input = None;
        self.output = None;
    }

    pub fn open_input(&mut self) -> Result<(), Error> {
        let pcm = PCM::new(&self.input_device, Direction::Capture, false)?;
        {
            let hwp = HwParams::any(&pcm)?;
            hwp.set_channels(2).expect("create_input failed to set channels");
            hwp.set_rate(48000, ValueOr::Nearest)?;
            hwp.set_format(Format::s16())?;
            hwp.set_access(Access::RWInterleaved)?;
            pcm.hw_params(&hwp)?;
        }
        self.input = Some(pcm);
        Ok(())
    }

    pub fn close_input(&mut self) ->  Result<(), Error> {
        self.input= None;
        Ok(())
    }

    pub fn open_output(&mut self) -> Result<(), Error> {
        let now = SystemTime::now();
        let pcm = PCM::new(&self.output_device, Direction::Playback, false)?;
        {
            let hwp = HwParams::any(&pcm)?;
            hwp.set_channels(2)?;
            hwp.set_rate(48000, ValueOr::Nearest)?;
            hwp.set_format(Format::s16())?;
            hwp.set_access(Access::RWInterleaved)?;
            pcm.hw_params(&hwp)?;
        }
        self.output = Some(pcm);
        Ok(())
    }

    pub fn close_output(&mut self) ->  Result<(), Error> {
        self.output = None;
        Ok(())
    }

    pub fn write_output(&mut self, buffer: &Vec<i16>) -> Result<(), Error> {
        match self.output.as_ref().expect("Could not access output to get delay").delay() {
            Ok(delay) => {
                let mut trim = 0;
                let max_delay = 2 * buffer.len() as i64;
                if delay > max_delay {
                    trim = delay - max_delay;
                }
                let io = self.output.as_ref().expect("failed to get io").io_i16()?;
                if trim > 0 {
                    let n = buffer.len() - trim as usize;
                    if n > 0  {
                        let trimmed_buffer = buffer[0..n].to_vec();
                        io.writei(&trimmed_buffer)?;
                    }
                } else {
                    io.writei(&buffer)?;
                }
            }
            Err(e) => {
                eprintln!("Failed to get delay: {}", e);
            }
        }

        Ok(())
    }

    fn list_pcm_devices(direction: Direction) -> Vec<String> {
        let mut devices = Vec::<String>::new();
        let hints = HintIter::new_str(None, "pcm").unwrap();
        for hint in hints {
            if hint.name.is_some() && hint.desc.is_some() && (hint.direction.is_none() || hint.direction.map(|dir| dir == direction).unwrap_or_default()) {
                devices.push(hint.name.expect("Error: cannot push name"));
            }
        }
        devices
    }

    pub fn configure(&mut self) -> (Grid, Label, Rc<RefCell<Audio>>) {

        let audio = Rc::new(RefCell::new(Audio {
            remote_input: self.remote_input,
            local_input: self.local_input,
            input_device: self.input_device.clone(),
            input: None,
            remote_output: self.remote_output,
            local_output: self.local_output,
            output_device: self.output_device.clone(),
            output: None,
        }));

        let label = Label::new(Some("Audio"));
        let grid = Grid::builder()
            .margin_start(0)
            .margin_end(0)
            .margin_top(0)
            .margin_bottom(0)
            .halign(Align::Center)
            .valign(Align::Center)
            .row_spacing(0)
            .column_spacing(0)
            .build();

        grid.set_column_homogeneous(true);
        grid.set_row_homogeneous(true);

        // build the UI
        let input_devices = Self::list_pcm_devices(Direction::Capture);

        let remote_input_check_button = CheckButton::with_label("Remote Input");
        remote_input_check_button.set_active(self.remote_input);
        grid.attach(&remote_input_check_button, 0, 0, 2, 1);
        let audio_clone = Rc::clone(&audio);
        remote_input_check_button.connect_toggled(move |button| {
            let is_active = button.is_active();
            audio_clone.borrow_mut().remote_input = is_active;
        });

        let local_input_check_button = CheckButton::with_label("Local Input");
        local_input_check_button.set_active(self.local_input);
        grid.attach(&local_input_check_button, 2, 0, 2, 1);
        let audio_clone = Rc::clone(&audio);
        local_input_check_button.connect_toggled(move |button| {
            let is_active = button.is_active();
            audio_clone.borrow_mut().local_input = is_active;
        });

        let input_combo_box = ComboBoxText::new();
        grid.attach(&input_combo_box, 4, 0, 5, 1);
        for i in 0..input_devices.len() {
            input_combo_box.append_text(&input_devices[i]);
            if input_devices[i] == self.input_device {
                input_combo_box.set_active(Some(i as u32));
            }
        }
	let audio_clone = Rc::clone(&audio);
	input_combo_box.connect_changed(move |combo_box| {
	    let input = combo_box.active_text();
            if let Some(input_string) = input { 
                audio_clone.borrow_mut().input_device = input_string.to_string();
            }
	});
       
        let output_devices = Self::list_pcm_devices(Direction::Playback);

        let remote_output_check_button = CheckButton::with_label("Remote Output");
        remote_output_check_button.set_active(self.remote_output);
        grid.attach(&remote_output_check_button, 0, 1, 2, 1);
        let audio_clone = Rc::clone(&audio);
        remote_output_check_button.connect_toggled(move |button| {
            let is_active = button.is_active();
            audio_clone.borrow_mut().remote_output = is_active;
        });

        let local_output_check_button = CheckButton::with_label("Local Output");
        local_output_check_button.set_active(self.local_output);
        grid.attach(&local_output_check_button, 2, 1, 2, 1);
        let audio_clone = Rc::clone(&audio);
        local_output_check_button.connect_toggled(move |button| {
            let is_active = button.is_active();
            audio_clone.borrow_mut().local_output = is_active;
        });

        let output_combo_box = ComboBoxText::new();
        grid.attach(&output_combo_box, 4, 1, 5, 1);
        for i in 0..output_devices.len() {
            output_combo_box.append_text(&output_devices[i]);
            if output_devices[i] == self.output_device {
                output_combo_box.set_active(Some(i as u32));
            }
        }
        let audio_clone = Rc::clone(&audio);
        output_combo_box.connect_changed(move |combo_box| {
            let output = combo_box.active_text();
            if let Some(output_string) = output { 
                audio_clone.borrow_mut().output_device = output_string.to_string();
            }
        });

        (grid, label, audio)
    }
}
