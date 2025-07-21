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

use alsa::Direction;
use gtk::prelude::*;
use gtk::{Adjustment, Align, ApplicationWindow, Builder, Button, CheckButton, ComboBoxText, DropDown, Grid, Label, Notebook, Orientation, PositionType, Scale, SpinButton, ToggleButton, Window};

use crate::radio::{Keyer, RadioMutex};
use crate::audio::*;

pub fn create_configure_dialog(parent: &ApplicationWindow, radio_mutex: &RadioMutex) -> Window {


    let ui_xml = include_str!("../ui/configure.xml");
    let builder = Builder::from_string(ui_xml);

    let window: Window = builder
            .object("configure_window")
            .expect("Could not get object `configure_window` from builder.");
    
    //window.set_transient_for(Some(parent));
    window.set_modal(true);

    let notebook: Notebook = builder
            .object("notebook")
            .expect("Could not get object `notebook` from builder.");

    // AUDIO

    let r = radio_mutex.radio.lock().unwrap();
        let remote_input = r.audio.remote_input;
        let local_input = r.audio.local_input;
        let input_device = r.audio.input_device.clone();
        let remote_output = r.audio.remote_output;
        let local_output = r.audio.local_output;
        let output_device = r.audio.output_device.clone();
    drop(r);

    let input_devices = Audio::list_pcm_devices(Direction::Capture);


    let remote_input_check_button: CheckButton = builder
            .object("remote_input_check_button")
            .expect("Could not get object `remote_input_check_button` from builder.");
    
    let remote_input_check_button = CheckButton::with_label("Remote Input");
    remote_input_check_button.set_active(remote_input);
    let radio_mutex_clone = radio_mutex.clone();
    remote_input_check_button.connect_toggled(move |button| {
        let is_active = button.is_active();
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.audio.remote_input = is_active;
    });

    let local_input_check_button: CheckButton = builder
            .object("local_input_check_button")
            .expect("Could not get object `local_input_check_button` from builder.");
    local_input_check_button.set_active(local_input);
    let radio_mutex_clone = radio_mutex.clone();
    local_input_check_button.connect_toggled(move |button| {
        let is_active = button.is_active();
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        if is_active {
            r.audio.open_input();
        }
        r.audio.local_input = is_active;
    });

    let input_combo_box: ComboBoxText = builder
            .object("input_combo_box")
            .expect("Could not get object `input_combo_box` from builder.");
    for i in 0..input_devices.len() {
        input_combo_box.append_text(&input_devices[i]);
        if input_devices[i] == input_device {
            input_combo_box.set_active(Some(i as u32));
        }
    }
    let radio_mutex_clone = radio_mutex.clone();
    input_combo_box.connect_changed(move |combo_box| {
        let input = combo_box.active_text();
        if let Some(input_string) = input {
            let mut r = radio_mutex_clone.radio.lock().unwrap();
            r.audio.input_device = input_string.to_string();
        }
    });

    let output_devices = Audio::list_pcm_devices(Direction::Playback);

    let remote_output_check_button: CheckButton = builder
            .object("remote_output_check_button")
            .expect("Could not get object `remote_output_check_button` from builder.");
    remote_output_check_button.set_active(remote_output);
    let radio_mutex_clone = radio_mutex.clone();
    remote_output_check_button.connect_toggled(move |button| {
        let is_active = button.is_active();
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.audio.remote_output = is_active;
    });

    let local_output_check_button: CheckButton = builder
            .object("local_output_check_button")
            .expect("Could not get object `local_output_check_button` from builder.");
    local_output_check_button.set_active(local_output);
    let radio_mutex_clone = radio_mutex.clone();
    local_output_check_button.connect_toggled(move |button| {
        let is_active = button.is_active();
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        if is_active {
            r.audio.open_output();
        }
        r.audio.local_output = is_active;
    });

    let output_combo_box: ComboBoxText = builder
            .object("output_combo_box")
            .expect("Could not get object `output_combo_box` from builder.");
    for i in 0..output_devices.len() {
        output_combo_box.append_text(&output_devices[i]);
        if output_devices[i] == output_device {
            output_combo_box.set_active(Some(i as u32));
        }
    }
    let radio_mutex_clone = radio_mutex.clone();
    output_combo_box.connect_changed(move |combo_box| {
        let output = combo_box.active_text();
        if let Some(output_string) = output {
            let mut r = radio_mutex_clone.radio.lock().unwrap();
            if r.audio.local_output {
                r.audio.close_output();
            }
            r.audio.output_device = output_string.to_string();
            if r.audio.local_output {
                r.audio.open_output();
            }
        }
     });

    let r = radio_mutex.radio.lock().unwrap();
    let mic_boost = r.mic_boost;
    let mic_ptt = r.mic_ptt;
    let mic_bias_ring = r.mic_bias_ring;
    let mic_bias_enable = r.mic_bias_enable;
    drop(r);

    let mic_boost_check_button: CheckButton = builder
            .object("mic_boost_check_button")
            .expect("Could not get object `mic_boost_check_button` from builder.");
    mic_boost_check_button.set_active(mic_boost);
    let radio_mutex_clone = radio_mutex.clone();
    mic_boost_check_button.connect_toggled(move |button| {
        let is_active = button.is_active();
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.mic_boost = is_active;
        r.updated = true;
    });

    let ptt_enable_check_button: CheckButton = builder
            .object("ptt_enable_check_button")
            .expect("Could not get object `ptt_enable_check_button` from builder.");
    ptt_enable_check_button.set_active(mic_ptt);
    let radio_mutex_clone = radio_mutex.clone();
    ptt_enable_check_button.connect_toggled(move |button| {
        let is_active = button.is_active();
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.mic_ptt = is_active;
        r.updated = true;
    });

    let mic_bias_ring_toggle_button: ToggleButton = builder
            .object("mic_bias_ring_toggle_button")
            .expect("Could not get object `mic_bias_ring_toggle_button` from builder.");
    mic_bias_ring_toggle_button.set_active(mic_bias_ring);
    let radio_mutex_clone = radio_mutex.clone();
    mic_bias_ring_toggle_button.connect_toggled(move |button| {
        let is_active = button.is_active();
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.mic_bias_ring = is_active;
        r.updated = true;
    });

    let mic_bias_tip_toggle_button: ToggleButton = builder
            .object("mic_bias_tip_toggle_button")
            .expect("Could not get object `mic_bias_tip_toggle_button` from builder.");
    mic_bias_tip_toggle_button.set_active(!mic_bias_ring);
    let radio_mutex_clone = radio_mutex.clone();
    mic_bias_tip_toggle_button.connect_toggled(move |button| {
        let is_active = button.is_active();
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.mic_bias_ring = !is_active;
        r.updated = true;
    });


    let mic_bias_enable_check_button: CheckButton = builder
            .object("mic_bias_enable_check_button")
            .expect("Could not get object `mic_bias_enable_check_button` from builder.");
    mic_bias_enable_check_button.set_active(mic_bias_enable);
    let radio_mutex_clone = radio_mutex.clone();
    mic_bias_enable_check_button.connect_toggled(move |button| {
        let is_active = button.is_active();
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.mic_bias_enable = is_active;
        r.updated = true;
    });

    // Display
    let r = radio_mutex.radio.lock().unwrap();
    let spectrum_average_time = r.receiver[0].spectrum_average_time;
    drop(r);
    let spectrum_average_adjustment: Adjustment = builder
            .object("spectrum_average_adjustment")
            .expect("Could not get object `spectrum_average_adjustment` from builder.");
    spectrum_average_adjustment.set_value(spectrum_average_time.into());
    let radio_mutex_clone = radio_mutex.clone();
    spectrum_average_adjustment.connect_value_changed(move |adjustment| {
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.receiver[0].spectrum_average_time = adjustment.value() as f32;
        r.receiver[0].update_spectrum_average(r.receiver[0].channel);
    }); 
    let r = radio_mutex.radio.lock().unwrap();
    let waterfall_average_time = r.receiver[0].waterfall_average_time;
    drop(r);
    let waterfall_average_adjustment: Adjustment = builder
            .object("waterfall_average_adjustment")
            .expect("Could not get object `waterfall_average_adjustment` from builder.");
    waterfall_average_adjustment.set_value(waterfall_average_time.into());
    let radio_mutex_clone = radio_mutex.clone();
    waterfall_average_adjustment.connect_value_changed(move |adjustment| {
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.receiver[0].waterfall_average_time = adjustment.value() as f32;
        r.receiver[0].update_waterfall_average(r.receiver[0].channel);
    }); 



    // CW

    let r = radio_mutex.radio.lock().unwrap();
        let cw_keyer_mode = r.cw_keyer_mode;
        let cw_keyer_internal = r.cw_keyer_internal;
        let cw_keys_reversed = r.cw_keys_reversed;
        let cw_breakin = r.cw_breakin;
    drop(r);

    let keyer_mode_dropdown: DropDown = builder
            .object("keyer_mode_dropdown")
            .expect("Could not get object `keyer_mode_dropdown` from builder.");
    keyer_mode_dropdown.set_selected(cw_keyer_mode.to_u32());
    let radio_mutex_clone = radio_mutex.clone();
    keyer_mode_dropdown.connect_selected_notify(move |dropdown| {
        let mode = dropdown.selected();
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.cw_keyer_mode = Keyer::from_u32(mode).expect("Invalid CW Keyer Mode");
        r.updated = true;
    });

    let cw_keyer_internal_check_button: CheckButton = builder
            .object("cw_keyer_internal_check_button")
            .expect("Could not get object `cw_keyer_internal_check_button` from builder.");
    cw_keyer_internal_check_button.set_active(cw_keyer_internal);
    let radio_mutex_clone = radio_mutex.clone();
    cw_keyer_internal_check_button.connect_toggled(move |button| {
        let is_active = button.is_active();
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.cw_keyer_internal = is_active;
        r.updated = true;
    });

    let cw_keys_reversed_check_button: CheckButton = builder
            .object("cw_keys_reversed_check_button")
            .expect("Could not get object `cw_keys_reversed_check_button` from builder.");
    cw_keys_reversed_check_button.set_active(cw_keys_reversed);
    let radio_mutex_clone = radio_mutex.clone();
    cw_keys_reversed_check_button.connect_toggled(move |button| {
        let is_active = button.is_active();
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.cw_keys_reversed = is_active;
        r.updated = true;
    });

    let cw_breakin_check_button: CheckButton = builder
            .object("cw_breakin_check_button")
            .expect("Could not get object `cw_breakin_check_button` from builder.");
    cw_breakin_check_button.set_active(cw_breakin);
    let radio_mutex_clone = radio_mutex.clone();
    cw_breakin_check_button.connect_toggled(move |button| {
        let is_active = button.is_active();
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.cw_breakin = is_active;
        r.updated = true;
    });


    let r = radio_mutex.radio.lock().unwrap();
    let rx = r.active_receiver;
    let enabled = r.receiver[rx].equalizer_enabled;
    let preamp = r.receiver[rx].equalizer_preamp as f64;
    let low = r.receiver[rx].equalizer_low as f64;
    let mid = r.receiver[rx].equalizer_mid as f64;
    let high = r.receiver[rx].equalizer_high as f64;
    drop(r);

    let equalizer_enabled_check_button: CheckButton = builder
            .object("equalizer_enabled_check_button")
            .expect("Could not get object `equalizer_enabled_check_button` from builder.");
    equalizer_enabled_check_button.set_active(enabled);
    let radio_mutex_clone = radio_mutex.clone();
    equalizer_enabled_check_button.connect_toggled(move |button| {
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        let rx = r.active_receiver;
        r.receiver[rx].equalizer_enabled = button.is_active();
        r.receiver[rx].enable_equalizer();
    });

    let preamp_scale: Scale = builder
            .object("preamp_scale")
            .expect("Could not get object `preamp_scale` from builder.");
    preamp_scale.add_mark(-12.0, PositionType::Left, Some("-12dB"));
    preamp_scale.add_mark(0.0, PositionType::Left, Some("0dB"));
    preamp_scale.add_mark(15.0, PositionType::Left, Some("15dB"));

    let low_scale: Scale = builder
            .object("low_scale")
            .expect("Could not get object `low_scale` from builder.");
    low_scale.add_mark(-12.0, PositionType::Left, Some("-12dB"));
    low_scale.add_mark(0.0, PositionType::Left, Some("0dB"));
    low_scale.add_mark(15.0, PositionType::Left, Some("15dB"));


    let mid_scale: Scale = builder
            .object("mid_scale")
            .expect("Could not get object `mid_scale` from builder.");
    mid_scale.add_mark(-12.0, PositionType::Left, Some("-12dB"));
    mid_scale.add_mark(0.0, PositionType::Left, Some("0dB"));
    mid_scale.add_mark(15.0, PositionType::Left, Some("15dB"));

    let high_scale: Scale = builder
            .object("high_scale")
            .expect("Could not get object `high_scale` from builder.");
    high_scale.add_mark(-12.0, PositionType::Left, Some("-12dB"));
    high_scale.add_mark(0.0, PositionType::Left, Some("0dB"));
    high_scale.add_mark(15.0, PositionType::Left, Some("15dB"));

    let preamp_adjustment: Adjustment = builder
            .object("preamp_adjustment")
            .expect("Could not get object `preamp_adjustment` from builder.");
    preamp_adjustment.set_value(preamp);
    let radio_mutex_clone = radio_mutex.clone();
    preamp_adjustment.connect_value_changed(move |adjustment| {
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        let rx = r.active_receiver;
        r.receiver[rx].equalizer_preamp = adjustment.value() as f32;
        r.receiver[rx].set_equalizer_values();
    });
    let low_adjustment: Adjustment = builder
            .object("low_adjustment")
            .expect("Could not get object `low_adjustment` from builder.");
    low_adjustment.set_value(low);
    let radio_mutex_clone = radio_mutex.clone();
    low_adjustment.connect_value_changed(move |adjustment| {
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        let rx = r.active_receiver;
        r.receiver[rx].equalizer_low = adjustment.value() as f32;
        r.receiver[rx].set_equalizer_values();
    });
    let mid_adjustment: Adjustment = builder
            .object("mid_adjustment")
            .expect("Could not get object `mid_adjustment` from builder.");
    mid_adjustment.set_value(mid);
    let radio_mutex_clone = radio_mutex.clone();
    mid_adjustment.connect_value_changed(move |adjustment| {
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        let rx = r.active_receiver;
        r.receiver[rx].equalizer_mid = adjustment.value() as f32;
        r.receiver[rx].set_equalizer_values();
    });
    let high_adjustment: Adjustment = builder
            .object("high_adjustment")
            .expect("Could not get object `high_adjustment` from builder.");
    high_adjustment.set_value(high);
    let radio_mutex_clone = radio_mutex.clone();
    high_adjustment.connect_value_changed(move |adjustment| {
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        let rx = r.active_receiver;
        r.receiver[rx].equalizer_high = adjustment.value() as f32;
        r.receiver[rx].set_equalizer_values();
    });

    let ok_button: Button = builder
            .object("ok_button")
            .expect("Could not get object `ok_button` from builder.");
    let window_for_ok = window.clone();
    ok_button.connect_clicked(move |_| {
        window_for_ok.close();
    });

    window
}

