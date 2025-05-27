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
use gtk::{Adjustment, Align, ApplicationWindow, Button, CellRendererText, ComboBox, DrawingArea, Frame, Grid, Label, ListStore, Orientation, Scale, ToggleButton};
use gtk::{EventController, EventControllerMotion, EventControllerScroll, EventControllerScrollFlags, GestureClick};
use gtk::glib::Propagation;
use gtk::cairo::{Context, LineCap, LineJoin, LinearGradient}; 
use gtk::gdk::{Cursor, Event, ModifierType}; 
use gdk_pixbuf::Pixbuf;
use gdk_pixbuf::Colorspace;
use glib::ControlFlow::Continue;
use pangocairo;

use std::cell::Cell;
use std::cell::RefCell;
use std::fmt::Write;
use std::mem::drop;
use std::os::raw::c_int;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use std::{
    env, fs,
    path::{PathBuf},
};
use serde::{Deserialize, Serialize};


use crate::discovery::Device;
use crate::bands::Bands;
use crate::bands::BandGrid;
use crate::bands::BandInfo;
use crate::modes::Modes;
use crate::modes::ModeGrid;
use crate::filters::FilterGrid;
use crate::agc::AGC;
use crate::receiver::Receiver;
use crate::configure::*;
use crate::wdsp::*;
use crate::protocol1::Protocol1;
use crate::protocol2::Protocol2;
use crate::audio::*;

#[derive(Clone)]
struct StepItem {
    id: f32,
    text: String,
}

#[derive(Clone)]
struct AGCItem {
    id: i32,
    text: String,
}

#[derive(Serialize, Deserialize)]
pub struct Radio {
    pub name: String,
    pub supported_receivers: u8,
    pub receivers: u8,
    pub receiver: Vec<Receiver>,
    pub band_info: Vec<BandInfo>,
    #[serde(skip_serializing, skip_deserializing)]
    pub s_meter_dbm: f64,
    #[serde(skip_serializing, skip_deserializing)]
    pub subrx_s_meter_dbm: f64,
    #[serde(skip_serializing, skip_deserializing)]
    pub mox: bool,
    #[serde(skip_serializing, skip_deserializing)]
    pub tun: bool,
    //#[serde(skip_serializing, skip_deserializing)]
    pub audio: Audio,
}

impl Radio {

    // currently only supports 1 receiver
    pub fn new(device: Device) -> Radio {
        let name = "?".to_string();
        let supported_receivers = device.supported_receivers;
        let receivers: u8 = 1;
        let mut receiver: Vec<Receiver> = Vec::new();
        let band_info = BandInfo::new();
        for i in 0..receivers {
            receiver.push(Receiver::new(i, &band_info));
        }
        let s_meter_dbm = -121.0;
        let subrx_s_meter_dbm = -121.0;
        let mox = false;
        let tun = false;
        let audio = Audio::new();

        let radio = Radio {
            name,
            supported_receivers,
            receivers,
            receiver,
            band_info,
            s_meter_dbm,
            subrx_s_meter_dbm,
            mox,
            tun,
            audio,
        };

        radio
    }

    pub fn init(&mut self) {
        self.s_meter_dbm = -121.0;
        self.subrx_s_meter_dbm = -121.0;
        self.mox = false;
        self.tun = false;
    }

    pub fn run(radio: &Arc<Mutex<Radio>>, main_window: &ApplicationWindow, device: Device) {

        let content = gtk::Box::new(Orientation::Vertical, 0);
        content.set_hexpand(true);
        content.set_vexpand(true);

        let provider = gtk::CssProvider::new();
        provider.load_from_data(
            ".vfo-a-label {
                font-family: FreeSans;
                font-size: 28px;
                color: green;
             }
            .vfo-b-label {
                font-family: FreeSans;
                font-size: 28px;
                color: orange;
             }
            .s-meter-label {
                font-family: FreeSans;
                font-size: 28px;
                color: red;
             }
            .active-button {
                color: orange;
                background-image: none;
             }
            .inactive-button {
                color: black;
                background-image: none;
             }
            .active-tx-button {
                color: red;
                background-image: none;
             }
            .inactive-tx-button {
                color: black;
                background-image: none;
             }
            .active-band-button {
                color: orange;
                border-radius: 5px;
                border-style: solid;
                border-width: 1px;
                padding-top: 0px;
                padding-right: 0px;
                padding-bottom: 0px;
                padding-left: 0px;
                font-family: FreeSans;
                font-size: small;
                margin-top: 0px;
                margin-bottom: 0px;
                min-height: 0px;
                background-image: none;
             }
             .inactive-band-button {
                color: black;
                border-radius: 5px;
                border-style: solid;
                border-width: 1px;
                padding-top: 0px;
                padding-right: 0px;
                padding-bottom: 0px;
                padding-left: 0px;
                font-family: FreeSans;
                font-size: small;
                margin-top: 0px;
                margin-bottom: 0px;
                min-height: 0px;
                background-image: none;
             }
             button.orange {
                 color: orange;
             }",
        );
        gtk::StyleContext::add_provider_for_display(
            &gtk::gdk::Display::default().unwrap(),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        let main_grid = Grid::builder()
            .margin_start(0)
            .margin_end(0)
            .margin_top(0)
            .margin_bottom(0)
            .halign(Align::Center)
            .valign(Align::Center)
            .row_spacing(0)
            .column_spacing(0)
            .build();

        main_grid.set_column_homogeneous(true);
        main_grid.set_row_homogeneous(true);
        main_grid.set_hexpand(true);
        main_grid.set_vexpand(true);

        // add the main window grid
        content.append(&main_grid);

        let mut grid_row = 0;

        let configure_button = Button::with_label("Configure");
        let main_window_for_configure = main_window.clone();
        //let band_info_for_configure = band_info.clone();
        let radio_for_configure = Arc::clone(&radio);
        configure_button.connect_clicked(move |_| {
            let main_window_for_dialog = main_window_for_configure.clone();
            //let band_info_for_dialog = band_info_for_configure.clone();
            let radio_for_dialog = Arc::clone(&radio_for_configure);
            let configure_dialog = create_configure_dialog(&main_window_for_dialog, &radio_for_dialog);
            configure_dialog.present();
        });

        main_grid.attach(&configure_button, 0, 0, 1, 1);

        let vfo_a_frame = Frame::new(Some("VFO A"));
        main_grid.attach(&vfo_a_frame, 1, 0, 2, 1);

        let vfo_a_frequency = Label::new(Some("00.000.000"));
        vfo_a_frequency.set_css_classes(&["vfo-a-label"]);

        vfo_a_frame.set_child(Some(&vfo_a_frequency));
        {
            let r = radio.lock().unwrap();
            if r.receiver[0].ctun {
                let formatted_value = format_u32_with_separators(r.receiver[0].ctun_frequency as u32);
                vfo_a_frequency.set_label(&formatted_value);
            } else {
                let formatted_value = format_u32_with_separators(r.receiver[0].frequency_a as u32);
                vfo_a_frequency.set_label(&formatted_value);
            }
        }

        let vfo_grid = Grid::builder()
            .margin_start(0)
            .margin_end(0)
            .margin_top(0)
            .margin_bottom(0)
            .halign(Align::Center)
            .valign(Align::Center)
            .row_spacing(2)
            .column_spacing(2)
            .build();
        vfo_grid.set_hexpand(true);
        vfo_grid.set_vexpand(true);
        main_grid.attach(&vfo_grid, 3, 0, 2, 1);

        let button_a_to_b = Button::with_label("A>B");
        vfo_grid.attach(&button_a_to_b, 0, 0, 1, 1);

        let button_b_to_a = Button::with_label("A<B");
        vfo_grid.attach(&button_b_to_a, 0, 1, 1, 1);

        let button_a_swap_b = Button::with_label("A<>B");
        vfo_grid.attach(&button_a_swap_b, 1, 0, 1, 1);

        let button_split = Button::with_label("SPLIT");
        vfo_grid.attach(&button_split, 1, 1, 1, 1);

        let button_subrx = ToggleButton::with_label("Subrx");
        {
            let r = radio.lock().unwrap();
            button_subrx.set_active(r.receiver[0].subrx);
            if r.receiver[0].subrx {
                button_subrx.add_css_class("active-button");
            } else {
                button_subrx.add_css_class("inactive-button");
            }
        }
        vfo_grid.attach(&button_subrx, 2, 0, 1, 1);

        let button_ctun = ToggleButton::with_label("CTUN");
        {
            let r = radio.lock().unwrap();
            button_ctun.set_active(r.receiver[0].ctun);
            if r.receiver[0].ctun {
                button_ctun.add_css_class("active-button");
            } else {
                button_ctun.add_css_class("inactive-button");
            }
        }
        vfo_grid.attach(&button_ctun, 2, 1, 1, 1);


        let vfo_b_frame = Frame::new(Some("VFO B"));
        main_grid.attach(&vfo_b_frame, 5, 0, 2, 1);

        let radio_for_ctun = Arc::clone(&radio);
        let vfo_a_frequency_for_ctun = vfo_a_frequency.clone();
        let button_subrx_for_ctun = button_subrx.clone();
        button_ctun.connect_clicked(move |button| {
            let mut r = radio_for_ctun.lock().unwrap();
            if r.receiver[0].ctun {
                r.receiver[0].ctun_frequency = 0.0;
                r.receiver[0].ctun = false;
                button.remove_css_class("active-button");
                button.add_css_class("inactive-button");
                r.receiver[0].set_ctun(false);
                let formatted_value = format_u32_with_separators(r.receiver[0].frequency_a as u32);
                vfo_a_frequency_for_ctun.set_label(&formatted_value);
            } else {
                r.receiver[0].ctun_frequency = r.receiver[0].frequency_a;
                r.receiver[0].ctun = true;
                button.remove_css_class("inactive-button");
                button.add_css_class("active-button");
                r.receiver[0].set_ctun(true);
            }
            //button_subrx_for_ctun.set_sensitive(r.receiver[0].ctun);
        });

        let vfo_b_frequency = Label::new(Some("14.150.000"));
        vfo_b_frequency.set_css_classes(&["vfo-b-label"]);
        vfo_b_frame.set_child(Some(&vfo_b_frequency));
        {
            let r = radio.lock().unwrap();
            let formatted_value = format_u32_with_separators(r.receiver[0].frequency_b as u32);
            vfo_b_frequency.set_label(&formatted_value);
        }

        let radio_for_subrx = Arc::clone(&radio); 
        let vfo_b_frequency_for_subrx = vfo_b_frequency.clone();
        button_subrx.connect_clicked(move |button| {
            let mut r = radio_for_subrx.lock().unwrap();
            if r.receiver[0].subrx {
                r.receiver[0].subrx = false;
                button.remove_css_class("active-button");
                button.add_css_class("inactive-button");
            } else {
                r.receiver[0].subrx = true;
                button.remove_css_class("inactive-button");
                button.add_css_class("active-button");
                r.receiver[0].frequency_b = r.receiver[0].frequency_a;
                r.receiver[0].set_subrx_frequency();
                let formatted_value = format_u32_with_separators(r.receiver[0].frequency_a as u32);
                vfo_b_frequency_for_subrx.set_label(&formatted_value);
            }
        });

        let radio_for_a_to_b = Arc::clone(&radio);
        let vfo_b_frequency_for_a_to_b = vfo_b_frequency.clone();
        button_a_to_b.connect_clicked(move |_| {
            let mut r = radio_for_a_to_b.lock().unwrap();
            if r.receiver[0].ctun {
                r.receiver[0].frequency_b = r.receiver[0].ctun_frequency;
            } else {
                r.receiver[0].frequency_b = r.receiver[0].frequency_a;
            }
            let formatted_value = format_u32_with_separators(r.receiver[0].frequency_b as u32);
            vfo_b_frequency_for_a_to_b.set_label(&formatted_value);
        });

        let radio_for_b_to_a = Arc::clone(&radio);
        let vfo_a_frequency_for_b_to_a = vfo_a_frequency.clone();
        button_b_to_a.connect_clicked(move |_| {
            let mut r = radio_for_b_to_a.lock().unwrap();
            if r.receiver[0].ctun {
                r.receiver[0].ctun_frequency = r.receiver[0].frequency_b;
                r.receiver[0].set_ctun_frequency();
            } else {
                r.receiver[0].frequency_a = r.receiver[0].frequency_b;
            }
            let formatted_value = format_u32_with_separators(r.receiver[0].frequency_a as u32);
            vfo_a_frequency_for_b_to_a.set_label(&formatted_value);
        }); 

        let radio_for_a_swap_b = Arc::clone(&radio);
        let vfo_a_frequency_for_a_swap_b = vfo_a_frequency.clone();
        let vfo_b_frequency_for_a_swap_b = vfo_b_frequency.clone();
        button_a_swap_b.connect_clicked(move |_| {
            let mut r = radio_for_a_swap_b.lock().unwrap();
            let temp_frequency = r.receiver[0].frequency_b;
            if r.receiver[0].ctun {
                r.receiver[0].frequency_b = r.receiver[0].ctun_frequency;
                r.receiver[0].ctun_frequency = temp_frequency;
                let formatted_value = format_u32_with_separators(r.receiver[0].ctun_frequency as u32);
                vfo_a_frequency_for_a_swap_b.set_label(&formatted_value);
                r.receiver[0].set_ctun_frequency();
            } else {
                r.receiver[0].frequency_b = r.receiver[0].frequency_a;
                r.receiver[0].frequency_a = temp_frequency;
                let formatted_value = format_u32_with_separators(r.receiver[0].frequency_a as u32);
                vfo_a_frequency_for_a_swap_b.set_label(&formatted_value);
            }
            let formatted_value = format_u32_with_separators(r.receiver[0].frequency_b as u32);
            vfo_b_frequency_for_a_swap_b.set_label(&formatted_value);
        });


        let tune_step_frame = Frame::new(Some("Tune Step"));
        main_grid.attach(&tune_step_frame, 7, 0, 2, 1);

        let step_model = ListStore::new(&[f32::static_type(), String::static_type()]);
        let step_items = vec![
            StepItem { id: 1.0, text: "1Hz".to_string() },
            StepItem { id: 10.0, text: "10Hz".to_string() },
            StepItem { id: 25.0, text: "25Hz".to_string() },
            StepItem { id: 50.0, text: "50Hz".to_string() },
            StepItem { id: 100.0, text: "100Hz".to_string() },
            StepItem { id: 250.0, text: "250Hz".to_string() },
            StepItem { id: 500.0, text: "500Hz".to_string() },
            StepItem { id: 1000.0, text: "1KHz".to_string() },
            StepItem { id: 5000.0, text: "5KHz".to_string() },
            StepItem { id: 9000.0, text: "9KHz".to_string() },
            StepItem { id: 10000.0, text: "10KHz".to_string() },
            StepItem { id: 100000.0, text: "100KHz".to_string() },
            StepItem { id: 250000.0, text: "250KHz".to_string() },
            StepItem { id: 500000.0, text: "500KHz".to_string() },
            StepItem { id: 1000000.0, text: "1MHz".to_string() },
        ];

        for item in step_items.iter() {
            let values: [(u32, &dyn ToValue); 2] = [
                (0, &item.id),
                (1, &item.text),
            ];
            step_model.set(&step_model.append(), &values);
        }

        let step_combo = ComboBox::with_model(&step_model);
        let step_renderer = CellRendererText::new();
        step_combo.pack_start(&step_renderer, true);
        step_combo.add_attribute(&step_renderer, "text", 1); // 1 is the index of the text column

        let r = radio.lock().unwrap();
        step_combo.set_active(Some(r.receiver[0].step_index as u32));
        drop(r);
        tune_step_frame.set_child(Some(&step_combo));

        let step_combo_radio = Arc::clone(&radio);
        step_combo.connect_changed(move |step_combo| {
            let mut r = step_combo_radio.lock().unwrap();
            let index = step_combo.active().unwrap_or(0);
            let mut step = 1000.0;
            match index {
                0 => step = 1.0,
                1 => step = 10.0,
                2 => step = 25.0,
                3 => step = 50.0,
                4 => step = 100.0,
                5 => step = 250.0,
                6 => step = 500.0,
                7 => step = 1000.0,
                8 => step = 5000.0,
                9 => step = 9000.0,
                10 => step = 10000.0,
                11 => step = 100000.0,
                12 => step = 250000.0,
                13 => step = 500000.0,
                14 => step = 1000000.0,
                _ => step = 1000.0,
            }
            r.receiver[0].step_index = index as usize;
            r.receiver[0].step = step;
        });


        let meter_frame = Frame::new(Some("S Meter"));
        let meter_display = DrawingArea::new();
        meter_frame.set_child(Some(&meter_display));
        main_grid.attach(&meter_frame, 9, 0, 2, 1);

        meter_display.connect_resize(move |_, width, height| {
            println!("meter_display resized to: {}x{}", width, height);
        });

        let radio_for_meter_draw = Arc::clone(&radio);
        meter_display.set_draw_func(move |_da, cr, width, _height| {
            let r = radio_for_meter_draw.lock().unwrap();
            cr.set_source_rgb (1.0, 1.0, 1.0);
            cr.paint().unwrap();
            if width >= 114 {
                draw_meter(cr, r.s_meter_dbm, false);
                if r.receiver[0].subrx {
                    draw_meter(cr, r.subrx_s_meter_dbm, true);
                }
            }
        });

        let middle_button_pressed = Rc::new(RefCell::new(false));

        let spectrum_display = DrawingArea::new();
        spectrum_display.set_hexpand(true);
        spectrum_display.set_vexpand(true);
        spectrum_display.set_content_width(1024);
        spectrum_display.set_content_height(250);

        let last_spectrum_x = Rc::new(Cell::new(0.0));

        let cursor_nsresize = Cursor::from_name("ns-resize", None);
        let motion_event_controller_spectrum = EventControllerMotion::new();
        spectrum_display.add_controller(motion_event_controller_spectrum.clone());
        let spectrum_display_for_motion_event = spectrum_display.clone();
        let last_spectrum_x_clone = last_spectrum_x.clone();
        motion_event_controller_spectrum.connect_motion(
            move |_controller, x, _y| {
                last_spectrum_x_clone.set(x);
                if x < 40.0 {
                    spectrum_display_for_motion_event.set_cursor(cursor_nsresize.as_ref());
                } else {
                    spectrum_display_for_motion_event.set_cursor(None); // default
                }
            }
        );

        let scroll_controller_spectrum = EventControllerScroll::new(
            EventControllerScrollFlags::VERTICAL | EventControllerScrollFlags::KINETIC
        );
        let radio_clone = Arc::clone(&radio);
        let fa = vfo_a_frequency.clone();
        let fb = vfo_b_frequency.clone();
        let last_spectrum_x_clone = last_spectrum_x.clone();
        let middle_button_state = middle_button_pressed.clone();
        let button_subrx_clone = button_subrx.clone();
        scroll_controller_spectrum.connect_scroll(move |controller, _dx, dy| {
            let r = radio_clone.lock().unwrap();
            let subrx = r.receiver[0].subrx;
            drop(r);
            if *middle_button_state.borrow() && subrx {
                let mut r = radio_clone.lock().unwrap();
                r.receiver[0].frequency_b = r.receiver[0].frequency_b - (r.receiver[0].step * dy as f32);
                let frequency_low = r.receiver[0].frequency_a - (r.receiver[0].sample_rate/2) as f32;
                let frequency_high = r.receiver[0].frequency_a + (r.receiver[0].sample_rate/2) as f32;
                if r.receiver[0].frequency_b < frequency_low {
                    r.receiver[0].frequency_b = frequency_low;
                } else if r.receiver[0].frequency_b > frequency_high {
                    r.receiver[0].frequency_b = frequency_high;
                }
                r.receiver[0].set_subrx_frequency();
                let formatted_value = format_u32_with_separators(r.receiver[0].frequency_b as u32);
                fb.set_label(&formatted_value);
            } else if last_spectrum_x_clone.get() < 40.0 {
                let mut r = radio_clone.lock().unwrap();
                let b = r.receiver[0].band.to_usize();
                if dy < 0.0 {
                    r.band_info[b].spectrum_low = r.band_info[b].spectrum_low + 1.0;
                    r.band_info[b].spectrum_high = r.band_info[b].spectrum_high + 1.0;
                } else {
                    r.band_info[b].spectrum_low = r.band_info[b].spectrum_low - 1.0;
                    r.band_info[b].spectrum_high = r.band_info[b].spectrum_high - 1.0;
                }
            } else {
                spectrum_waterfall_scroll(&radio_clone, &fa, dy, &button_subrx_clone);
            }
            Propagation::Proceed
        });
        spectrum_display.add_controller(scroll_controller_spectrum.clone());

        let scroll_controller_a = EventControllerScroll::new(
            EventControllerScrollFlags::VERTICAL
        );
        let radio_clone = Arc::clone(&radio);
        let fa = vfo_a_frequency.clone();
        let button_subrx_clone = button_subrx.clone();
        scroll_controller_a.connect_scroll(move |controller, dx, dy| {
            spectrum_waterfall_scroll(&radio_clone, &fa, dy, &button_subrx_clone);
            Propagation::Proceed
        });
        vfo_a_frequency.add_controller(scroll_controller_a);

        let scroll_controller_b = EventControllerScroll::new(
            EventControllerScrollFlags::VERTICAL
        );
        let radio_clone = Arc::clone(&radio);
        let f = vfo_b_frequency.clone();
        scroll_controller_b.connect_scroll(move |_controller, _dx, dy| {
            let mut r = radio_clone.lock().unwrap();
            r.receiver[0].frequency_b = r.receiver[0].frequency_b - (r.receiver[0].step * dy as f32);
            if r.receiver[0].subrx {
                let frequency_low = r.receiver[0].frequency_a - (r.receiver[0].sample_rate/2) as f32;
                let frequency_high = r.receiver[0].frequency_a + (r.receiver[0].sample_rate/2) as f32;
                if r.receiver[0].frequency_b < frequency_low {
                    r.receiver[0].frequency_b = frequency_low;
                } else if r.receiver[0].frequency_b > frequency_high {
                    r.receiver[0].frequency_b = frequency_high;
                }
                r.receiver[0].set_subrx_frequency();
            }
            let formatted_value = format_u32_with_separators(r.receiver[0].frequency_b as u32);
            f.set_label(&formatted_value);
            Propagation::Proceed
        });
        vfo_b_frequency.add_controller(scroll_controller_b);

        let spectrum_click_gesture = Rc::new(GestureClick::new());
        spectrum_click_gesture.set_button(0); // all buttons
        let spectrum_click_gesture_clone_for_callback = spectrum_click_gesture.clone();
        let radio_clone = Arc::clone(&radio);
        let fa = vfo_a_frequency.clone();
        let fb = vfo_b_frequency.clone();
        let press_state = middle_button_pressed.clone();
        let button_subrx_clone = button_subrx.clone();
        spectrum_click_gesture_clone_for_callback.connect_pressed(move |gesture, _, x, _y| {
            let da = gesture.widget().unwrap();
            let width = da.allocated_width();
            if gesture.current_button() == 2 { // middle button
                *press_state.borrow_mut() = true;
            } else {
                spectrum_waterfall_clicked(&radio_clone, &fa, &fb, x, width, gesture.current_button(), &button_subrx_clone);
            }
        });
        let press_state = middle_button_pressed.clone();
        spectrum_click_gesture_clone_for_callback.connect_released(move |gesture, _, x, _y| {
            if gesture.current_button() == 2 { // middle button
                *press_state.borrow_mut() = false;
            }
        });

        spectrum_display.add_controller(<GestureClick as Clone>::clone(&spectrum_click_gesture).upcast::<EventController>());

        spectrum_display.connect_resize(move |_, width, height| {
            println!("Spectrum resized to: {}x{}", width, height);
        });

        main_grid.attach(&spectrum_display, 1, 1, 10, 3);


        let pixbuf: Rc<RefCell<Option<Pixbuf>>> = Rc::new(RefCell::new(None));
        let waterfall_display = DrawingArea::new();
        waterfall_display.set_hexpand(true);
        waterfall_display.set_vexpand(true);
        waterfall_display.set_content_width(1024);
        waterfall_display.set_content_height(250);

        let last_waterfall_x = Rc::new(Cell::new(0.0));

        let cursor_nsresize = Cursor::from_name("ns-resize", None);
        let motion_event_controller_waterfall = EventControllerMotion::new();
        waterfall_display.add_controller(motion_event_controller_waterfall.clone());
        let waterfall_display_for_motion_event = waterfall_display.clone();
        let last_waterfall_x_clone = last_waterfall_x.clone();
        motion_event_controller_waterfall.connect_motion(
            move |_controller, x, _y| {
                last_waterfall_x_clone.set(x);
                if x < 40.0 {
                    waterfall_display_for_motion_event.set_cursor(cursor_nsresize.as_ref());
                } else {
                    waterfall_display_for_motion_event.set_cursor(None); // default
                }
            }
        );

        let scroll_controller_waterfall = EventControllerScroll::new(
            EventControllerScrollFlags::VERTICAL
        );
        let radio_clone = Arc::clone(&radio);
        let fa = vfo_a_frequency.clone();
        let last_waterfall_x_clone = last_waterfall_x.clone();
        let button_subrx_clone = button_subrx.clone();
        scroll_controller_waterfall.connect_scroll(move |_controller, _dx, dy| {
            if last_waterfall_x_clone.get() < 40.0 {
                // adjust spectrum low and high
                let mut r = radio_clone.lock().unwrap();
                let b = r.receiver[0].band.to_usize();
                if dy < 0.0 {
                    r.band_info[b].waterfall_low = r.band_info[b].waterfall_low + 1.0;
                    r.band_info[b].waterfall_high = r.band_info[b].waterfall_high + 1.0;
                } else {
                    r.band_info[b].waterfall_low = r.band_info[b].waterfall_low - 1.0;
                    r.band_info[b].waterfall_high = r.band_info[b].waterfall_high - 1.0;
                }
            } else {
                spectrum_waterfall_scroll(&radio_clone, &fa, dy, &button_subrx_clone);
            }
            Propagation::Proceed
        });
        waterfall_display.add_controller(scroll_controller_waterfall.clone());

        let waterfall_click_gesture = Rc::new(GestureClick::new());
        waterfall_click_gesture.set_button(0); // all buttons
        let waterfall_click_gesture_clone_for_callback = spectrum_click_gesture.clone();
        let waterfall_click_gesture_clone_for_callback = waterfall_click_gesture.clone();
        let radio_clone = Arc::clone(&radio);
        let fa = vfo_a_frequency.clone();
        let fb = vfo_b_frequency.clone();
        let button_subrx_clone = button_subrx.clone();
        waterfall_click_gesture_clone_for_callback.connect_pressed(move |gesture, _, x, _y| {
            let da = gesture.widget().unwrap();
            let width = da.allocated_width();
            spectrum_waterfall_clicked(&radio_clone, &fa, &fb, x, width, gesture.current_button(), &button_subrx_clone);
        });
        waterfall_display.add_controller(<GestureClick as Clone>::clone(&waterfall_click_gesture).upcast::<EventController>());

        let radio_clone = Arc::clone(&radio);
        waterfall_display.connect_resize(move |_, width, height| {
            println!("Waterfall resized to: {}x{}", width, height);
            let mut r = radio_clone.lock().unwrap();
            r.receiver[0].spectrum_width = width;
            r.receiver[0].init_analyzer(r.receiver[0].channel);
        });
        main_grid.attach(&waterfall_display, 1, 4, 10, 3);

        let band_frame = Frame::new(Some("Band"));
        main_grid.attach(&band_frame, 11, 0, 2, 2);
        
        let radio_clone = Arc::clone(&radio);
        let band_grid = Rc::new(RefCell::new(BandGrid::new(radio_clone)));
        let band_grid_ref = band_grid.borrow().get_widget().clone();
        band_frame.set_child(Some(&band_grid_ref));

        let mode_frame = Frame::new(Some("Mode"));
        main_grid.attach(&mode_frame, 11, 2, 2, 2);
        let mode_grid = Rc::new(RefCell::new(ModeGrid::new()));
        let mode_grid_ref = mode_grid.borrow().get_widget().clone();
        mode_frame.set_child(Some(&mode_grid_ref));

        let filter_frame = Frame::new(Some("Filter"));
        main_grid.attach(&filter_frame, 11, 4, 2, 2);
        let filter_grid = Rc::new(RefCell::new(FilterGrid::new()));
        let filter_grid_ref = filter_grid.borrow().get_widget().clone();
        filter_frame.set_child(Some(&filter_grid_ref));

        let mut band_grid_for_callback = band_grid.borrow_mut();
        let mode_grid_for_callback = mode_grid.clone();
        let filter_grid_for_callback = filter_grid.clone();
        let radio_for_callback = Arc::clone(&radio);
        let vfo_a_frequency_for_callback = vfo_a_frequency.clone();
        let r = radio.lock().unwrap();
        let band = r.receiver[0].band.to_usize();
        drop(r);
        let button_subrx_clone = button_subrx.clone();
        band_grid_for_callback.set_callback(move|index| {
            // save current band info
            let mut r = radio_for_callback.lock().unwrap();
            let b = r.receiver[0].band.to_usize();
            r.band_info[b].current = r.receiver[0].frequency_a;
            // get new band info
            r.receiver[0].band = Bands::from_usize(index).expect("invalid band index");
            r.receiver[0].frequency_a = r.band_info[index].current;
            if r.receiver[0].ctun {
                r.receiver[0].ctun_frequency = r.receiver[0].frequency_a;
                r.receiver[0].set_ctun_frequency();
            }

            if !r.receiver[0].filters_manual {
                r.receiver[0].filters = r.band_info[index].filters;
            }

            if r.receiver[0].subrx {
                r.receiver[0].subrx = false;
                button_subrx_clone.set_active(r.receiver[0].subrx);
            }

            let filter_grid_mut = filter_grid_for_callback.borrow_mut();
            let mode_grid_mut = mode_grid_for_callback.borrow_mut();
            mode_grid_mut.set_active_index(r.band_info[index].mode.to_usize());
            filter_grid_mut.update_filter_buttons(r.band_info[index].mode.to_usize());
            filter_grid_mut.set_active_index(r.band_info[index].filter.to_usize());
            r.receiver[0].mode = r.band_info[index].mode.to_usize();
            let (low, high) = filter_grid_mut.get_filter_values(r.band_info[index].mode.to_usize(), r.band_info[index].filter.to_usize());

            r.receiver[0].filter_low = low;
            r.receiver[0].filter_high = high;

            if r.receiver[0].mode == Modes::CWL.to_usize() {
                r.receiver[0].filter_low = -r.receiver[0].cw_sidetone - low;
                r.receiver[0].filter_high = -r.receiver[0].cw_sidetone + high;
            } else if r.receiver[0].mode == Modes::CWU.to_usize() {
                r.receiver[0].filter_low = r.receiver[0].cw_sidetone - low;
                r.receiver[0].filter_high = r.receiver[0].cw_sidetone + high;
            }
            r.receiver[0].set_mode();

            let formatted_value = format_u32_with_separators(r.receiver[0].frequency_a as u32);
            vfo_a_frequency_for_callback.set_label(&formatted_value);
        }, band);

        let r = radio.lock().unwrap();
        let mode = r.receiver[0].mode;
        drop(r);

        let radio_for_mode_set_callback = Arc::clone(&radio);
        let filter_grid_for_mode_set_callback = filter_grid.clone();
        let mut mode_grid_for_callback = mode_grid.borrow_mut();
        mode_grid_for_callback.set_callback(move|index| {
            let mut r = radio_for_mode_set_callback.lock().unwrap();
            r.receiver[0].mode = index;
            let filter_grid = filter_grid_for_mode_set_callback.borrow_mut();
            filter_grid.update_filter_buttons(index);

            let (low, high) = filter_grid.get_filter_values(index, r.receiver[0].filter);
            r.receiver[0].filter_low = low;
            r.receiver[0].filter_high = high;

            if r.receiver[0].mode == Modes::CWL.to_usize() {
                r.receiver[0].filter_low = -r.receiver[0].cw_sidetone - low;
                r.receiver[0].filter_high = -r.receiver[0].cw_sidetone + high;
            } else if r.receiver[0].mode == Modes::CWU.to_usize() {
                r.receiver[0].filter_low = r.receiver[0].cw_sidetone - low;
                r.receiver[0].filter_high = r.receiver[0].cw_sidetone + high;
            }
            r.receiver[0].set_mode();
        }, mode);

        let r = radio.lock().unwrap();
        let filter = r.receiver[0].filter;
        drop(r);

        let radio_for_filter_callback = Arc::clone(&radio);
        let filter_grid_for_set_callback = filter_grid.clone();
        let mut filter_grid_for_callback = filter_grid.borrow_mut();
        filter_grid_for_callback.set_callback(move|index| {
            let mut r = radio_for_filter_callback.lock().unwrap();
            let filter_grid = filter_grid_for_set_callback.borrow();
            r.receiver[0].filter = index;
            let (low, high) = filter_grid.get_filter_values(r.receiver[0].mode, r.receiver[0].filter);
            r.receiver[0].filter_low = low;
            r.receiver[0].filter_high = high;

            if r.receiver[0].mode == Modes::CWL.to_usize() {
                r.receiver[0].filter_low = -r.receiver[0].cw_sidetone - low;
                r.receiver[0].filter_high = -r.receiver[0].cw_sidetone + high;
            } else if r.receiver[0].mode == Modes::CWU.to_usize() {
                r.receiver[0].filter_low = r.receiver[0].cw_sidetone - low;
                r.receiver[0].filter_high = r.receiver[0].cw_sidetone + high;
            }
            r.receiver[0].set_filter();
        }, filter);

        let tx_grid = Grid::builder()
            .margin_start(0)
            .margin_end(0)
            .margin_top(0)
            .margin_bottom(0)
            .halign(Align::Center)
            .valign(Align::Center)
            .row_spacing(2)
            .column_spacing(2)
            .build();
        grid_row = grid_row + 1;
        main_grid.attach(&tx_grid, 0, grid_row, 1, 1);

        let button_mox = ToggleButton::with_label("MOX");
        {
            let r = radio.lock().unwrap();
            if r.mox {
                button_mox.add_css_class("active-tx-button");
            } else {
                button_mox.add_css_class("inactive-tx-button");
            }
        }
        tx_grid.attach(&button_mox, 0, 0, 1, 1);

        let button_tun = ToggleButton::with_label("TUN");
        {
            let r = radio.lock().unwrap();
            if r.tun {
                button_tun.add_css_class("active-tx-button");
            } else {
                button_tun.add_css_class("inactive-tx-button");
            }
        }
        tx_grid.attach(&button_tun, 0, 1, 1, 1);

        let mox_radio = Arc::clone(&radio);
        let tun_button_for_mox = button_tun.clone();
        button_mox.connect_clicked(move |button| {
            let mut r = mox_radio.lock().unwrap();
            r.mox = button.is_active();
            if r.mox {
                button.remove_css_class("inactive-tx-button");
                button.add_css_class("active-tx-button");
                if tun_button_for_mox.is_active() {
                   tun_button_for_mox.set_active(false);
                   tun_button_for_mox.remove_css_class("active-tx-button");
                   tun_button_for_mox.add_css_class("inactive-tx-button");
                   r.tun = false;
                }
            } else {
                button.remove_css_class("active-tx-button");
                button.add_css_class("inactive-tx-button");
            }
        });

        let tun_radio = Arc::clone(&radio);
        let mox_button_for_tun = button_mox.clone();
        button_tun.connect_clicked(move |button| {
            let mut r = tun_radio.lock().unwrap();
            r.tun = button.is_active();
            if r.tun {
                button.remove_css_class("inactive-tx-button");
                button.add_css_class("active-tx-button");
                if mox_button_for_tun.is_active() {
                   mox_button_for_tun.set_active(false);
                   mox_button_for_tun.remove_css_class("active-tx-button");
                   mox_button_for_tun.add_css_class("inactive-tx-button");
                   r.mox = false;
                }
            } else {
                button.remove_css_class("active-tx-button");
                button.add_css_class("inactive-tx-button");
            }
        });

        
        let afgain_frame = Frame::new(Some("AF Gain"));
        grid_row = grid_row + 1;
        main_grid.attach(&afgain_frame, 0, grid_row, 1, 1);
        let afgain_adjustment = Adjustment::new(
            0.0, //(r.receiver[0].afgain * 100.0).into(), // Initial value
            0.0,  // Minimum value
            100.0, // Maximum value
            1.0,  // Step increment
            1.0, // Page increment
            0.0,  // Page size (not typically used for simple scales)
        );
        let afgain_scale = Scale::new(Orientation::Horizontal, Some(&afgain_adjustment));
        afgain_scale.set_digits(0); // Display whole numbers
        afgain_scale.set_draw_value(true); // Display the current value next to the slider
        {
            let r = radio.lock().unwrap();
            afgain_adjustment.set_value((r.receiver[0].afgain *100.0).into());
        }
        afgain_frame.set_child(Some(&afgain_scale));

        let afgain_radio = Arc::clone(&radio);
        afgain_adjustment.connect_value_changed(move |adjustment| {
            let mut r = afgain_radio.lock().unwrap();
            r.receiver[0].afgain = (adjustment.value() / 100.0) as f32;
            r.receiver[0].set_afgain();
        });

        let afpan_frame = Frame::new(Some("AF Pan"));
        grid_row = grid_row + 1;
        main_grid.attach(&afpan_frame, 0, grid_row, 1, 1);
        let afpan_adjustment = Adjustment::new(
            0.0, //(r.receiver[0].afpan * 100.0).into(), // Initial value
            0.0,  // Minimum value
            100.0, // Maximum value
            1.0,  // Step increment
            1.0, // Page increment
            0.0,  // Page size (not typically used for simple scales)
        );
        let afpan_scale = Scale::new(Orientation::Horizontal, Some(&afpan_adjustment));
        afpan_scale.set_digits(0); // Display whole numbers
        afpan_scale.set_draw_value(true); // Display the current value next to the slider
        {
            let r = radio.lock().unwrap();
            afpan_adjustment.set_value((r.receiver[0].afpan *100.0).into());
        }
        afpan_frame.set_child(Some(&afpan_scale));

        let afpan_radio = Arc::clone(&radio);
        afpan_adjustment.connect_value_changed(move |adjustment| {
            let mut r = afpan_radio.lock().unwrap();
            r.receiver[0].afpan = (adjustment.value() / 100.0) as f32;
            r.receiver[0].set_afpan();
        });


        let agc_frame = Frame::new(Some("AGC"));
        grid_row = grid_row + 1;
        main_grid.attach(&agc_frame, 0, grid_row, 1, 1);

        let agc_model = ListStore::new(&[f32::static_type(), String::static_type()]);
        let agc_items = vec![
            AGCItem { id: 0, text: "Off".to_string() },
            AGCItem { id: 1, text: "Long".to_string() },
            AGCItem { id: 2, text: "Slow".to_string() },
            AGCItem { id: 3, text: "Med".to_string() },
            AGCItem { id: 4, text: "Fast".to_string() },
        ];

        for item in agc_items.iter() {
            let values: [(u32, &dyn ToValue); 2] = [
                (0, &item.id),
                (1, &item.text),
            ];
            agc_model.set(&agc_model.append(), &values);
        }

        let agc_combo = ComboBox::with_model(&agc_model);
        let agc_renderer = CellRendererText::new();
        agc_combo.pack_start(&agc_renderer, true);
        agc_combo.add_attribute(&agc_renderer, "text", 1);
        {
            let r = radio.lock().unwrap();
            agc_combo.set_active(Some(r.receiver[0].agc as u32));
        }
        agc_frame.set_child(Some(&agc_combo));

        let agc_combo_radio = Arc::clone(&radio);
        agc_combo.connect_changed(move |agc_combo| {
            let mut r = agc_combo_radio.lock().unwrap();
            let index = agc_combo.active().unwrap_or(0);
            r.receiver[0].agc = AGC::from_i32(index as i32).expect("Invalid AGC");
            AGC::set_agc(&r.receiver[0], r.receiver[0].channel);
        });

        let agcgain_frame = Frame::new(Some("AGC Gain"));
        grid_row = grid_row + 1;
        main_grid.attach(&agcgain_frame, 0, grid_row, 1, 1);
        let agcgain_adjustment = Adjustment::new(
            0.0, // r.receiver[0].agcgain.into(), // Initial value
            -20.0,  // Minimum value
            120.0, // Maximum value
            1.0,  // Step increment
            1.0, // Page increment
            0.0,  // Page size (not typically used for simple scales)
        );
        let agcgain_scale = Scale::new(Orientation::Horizontal, Some(&agcgain_adjustment));
        agcgain_scale.set_digits(0); // Display whole numbers
        agcgain_scale.set_draw_value(true); // Display the current value next to the slider
        {
            let r = radio.lock().unwrap();
            agcgain_adjustment.set_value(r.receiver[0].agcgain.into());
        }
        agcgain_frame.set_child(Some(&agcgain_scale));

        let agcgain_radio = Arc::clone(&radio);
        agcgain_adjustment.connect_value_changed(move |adjustment| {
            let mut r = agcgain_radio.lock().unwrap();
            r.receiver[0].agcgain = adjustment.value() as f32;
            r.receiver[0].set_agcgain();
        });

        if device.device == 6 { // HERMES LITE
            let rxgain_frame = Frame::new(Some("RX Gain"));
            grid_row = grid_row + 1;
            main_grid.attach(&rxgain_frame, 0, grid_row, 1, 1); 
            let rxgain_adjustment = Adjustment::new(
                0.0, //r.receiver[0].rxgain.into(), // Initial value
                -12.0,  // Minimum value
                48.0, // Maximum value
                1.0,  // Step increment
                1.0, // Page increment
                0.0,  // Page size (not typically used for simple scales)
            );
            let rxgain_scale = Scale::new(Orientation::Horizontal, Some(&rxgain_adjustment));
            rxgain_scale.set_digits(0); // Display whole numbers
            rxgain_scale.set_draw_value(true); // Display the current value next to the slider
            {
                let r = radio.lock().unwrap();
                rxgain_adjustment.set_value(r.receiver[0].rxgain.into());
            }
            rxgain_frame.set_child(Some(&rxgain_scale));

            let rxgain_radio = Arc::clone(&radio);
            rxgain_adjustment.connect_value_changed(move |adjustment| {
                let mut r = rxgain_radio.lock().unwrap();
                r.receiver[0].rxgain = adjustment.value() as i32;
            });
/*
        } else {
            let adcattn_frame = Frame::new(Some("ADC Attn"));
            grid_row = grid_row + 1;
            main_grid.attach(&adcattn_frame, 0, grid_row, 1, 1);  
            let adcattn_adjustment = Adjustment::new(
                r.receiver[0].attenuation.into(), // Initial value
                0.0,  // Minimum value
                31.0, // Maximum value
                1.0,  // Step increment
                1.0, // Page increment
                0.0,  // Page size (not typically used for simple scales)
            );
            let adcattn_scale = Scale::new(Orientation::Horizontal, Some(&adcattn_adjustment));
            adcattn_scale.set_digits(0); // Display whole numbers
            adcattn_scale.set_draw_value(true); // Display the current value next to the slider
            adcattn_frame.set_child(Some(&adcattn_scale));

            let adcattn_radio = Arc::clone(&radio);
            adcattn_adjustment.connect_value_changed(move |adjustment| {
                let mut r = adcattn_radio.lock().unwrap();
                r.receiver[0].attenuation = adjustment.value() as i32;
            });
*/
        }


        let micgain_frame = Frame::new(Some("Mic Gain"));
        grid_row = grid_row + 1;
        main_grid.attach(&micgain_frame, 0, grid_row, 1, 1);
        let micgain_adjustment = Adjustment::new(
            50.0, // Initial value
            0.0,  // Minimum value
            100.0, // Maximum value
            1.0,  // Step increment
            1.0, // Page increment
            0.0,  // Page size (not typically used for simple scales)
        );
        let micgain_scale = Scale::new(Orientation::Horizontal, Some(&micgain_adjustment));
        micgain_scale.set_digits(0); // Display whole numbers
        micgain_scale.set_draw_value(true); // Display the current value next to the slider
        micgain_frame.set_child(Some(&micgain_scale));

        let drive_frame = Frame::new(Some("TX Drive"));
        grid_row = grid_row + 1;
        main_grid.attach(&drive_frame, 0, grid_row, 1, 1);
        let drive_adjustment = Adjustment::new(
            50.0, // Initial value
            0.0,  // Minimum value
            100.0, // Maximum value
            1.0,  // Step increment
            1.0, // Page increment
            0.0,  // Page size (not typically used for simple scales)
        );
        let drive_scale = Scale::new(Orientation::Horizontal, Some(&drive_adjustment));
        drive_scale.set_digits(0); // Display whole numbers
        drive_scale.set_draw_value(true); // Display the current value next to the slider
        drive_frame.set_child(Some(&drive_scale));

        let noise_grid = Grid::builder()
            .margin_start(0)
            .margin_end(0)
            .margin_top(0)
            .margin_bottom(0)
            .halign(Align::Center)
            .valign(Align::Center)
            .row_spacing(2)
            .column_spacing(2)
            .build();
        main_grid.attach(&noise_grid, 1, 7, 2, 1);

        let button_nr = ToggleButton::with_label("NR2");
        {
            let r = radio.lock().unwrap();
            button_nr.set_active(r.receiver[0].nr);
        }
        noise_grid.attach(&button_nr, 0, 0, 1, 1);
        let radio_for_nr = Arc::clone(&radio);
        button_nr.connect_clicked(move |_| {
            let mut r = radio_for_nr.lock().unwrap();
            if r.receiver[0].nr {
                r.receiver[0].nr = false;
            } else {
                r.receiver[0].nr = true;
            }
            r.receiver[0].set_nr();
        });

        let button_nb = ToggleButton::with_label("NB2");
        {
            let r = radio.lock().unwrap();
            button_nb.set_active(r.receiver[0].nb);
        }
        noise_grid.attach(&button_nb, 0, 1, 1, 1);
        let radio_for_nb = Arc::clone(&radio);
        button_nb.connect_clicked(move |_| {
            let mut r = radio_for_nb.lock().unwrap();
            if r.receiver[0].nb {
                r.receiver[0].nb = false;
            } else {
                r.receiver[0].nb = true;
            }
            r.receiver[0].set_nb();
        });

        let button_anf = ToggleButton::with_label("ANF");
        {
            let mut r = radio.lock().unwrap();
            button_anf.set_active(r.receiver[0].anf);
        }
        noise_grid.attach(&button_anf, 1, 0, 1, 1);
        let radio_for_anf = Arc::clone(&radio);
        button_anf.connect_clicked(move |_| {
            let mut r = radio_for_anf.lock().unwrap();
            if r.receiver[0].anf {
                r.receiver[0].anf = false;
            } else { 
                r.receiver[0].anf = true;
            }
            r.receiver[0].set_anf();
        });

        let button_snb = ToggleButton::with_label("SNB");
        {
            let r = radio.lock().unwrap();
            button_snb.set_active(r.receiver[0].snb);
        }
        noise_grid.attach(&button_snb, 1, 1, 1, 1);
        let radio_for_snb = Arc::clone(&radio);
        button_snb.connect_clicked(move |_| {
            let mut r = radio_for_snb.lock().unwrap();
            if r.receiver[0].snb {
                r.receiver[0].snb = false;
            } else {
                r.receiver[0].snb = true;
            }
            r.receiver[0].set_snb();
        });

        let pan_frame = Frame::new(Some("Pan"));
        main_grid.attach(&pan_frame, 6, 7, 3, 1);
        let pan_adjustment = Adjustment::new(
            0.0, // r.receiver[0].pan.into(),
            0.0,  // Minimum value
            100.0, // Maximum value
            1.0,  // Step increment
            1.0, // Page increment
            0.0,  // Page size (not typically used for simple scales)
        );
        let pan_scale = Scale::new(Orientation::Horizontal, Some(&pan_adjustment));
        pan_scale.set_digits(0); // Display whole numbers
        pan_scale.set_draw_value(true); // Display the current value next to the slider
        {
            let r = radio.lock().unwrap();
            pan_adjustment.set_value(r.receiver[0].pan.into());
        }
        pan_frame.set_child(Some(&pan_scale));

        let pan_radio = Arc::clone(&radio);
        pan_adjustment.connect_value_changed(move |adjustment| {
            let mut r = pan_radio.lock().unwrap();
            if r.receiver[0].zoom > 1 {
                r.receiver[0].pan = adjustment.value() as i32;
            } else {
                r.receiver[0].pan = 0;
            }
        });

        let zoom_frame = Frame::new(Some("Zoom"));
        main_grid.attach(&zoom_frame, 3, 7, 3, 1);
        let zoom_adjustment = Adjustment::new(
            1.0, //r.receiver[0].zoom.into(),
            1.0,  // Minimum value
            16.0, // Maximum value
            1.0,  // Step increment
            1.0, // Page increment
            0.0,  // Page size (not typically used for simple scales)
        );
        let zoom_scale = Scale::new(Orientation::Horizontal, Some(&zoom_adjustment));
        zoom_scale.set_digits(0); // Display whole numbers
        zoom_scale.set_draw_value(true); // Display the current value next to the slider
        {
            let r = radio.lock().unwrap();
            zoom_adjustment.set_value(r.receiver[0].zoom.into());
        }
        zoom_frame.set_child(Some(&zoom_scale));

        let zoom_radio = Arc::clone(&radio);
        zoom_adjustment.connect_value_changed(move |adjustment| {
            let mut r = zoom_radio.lock().unwrap();
            r.receiver[0].zoom = adjustment.value() as i32;
            r.receiver[0].init_analyzer(r.receiver[0].channel);
            if adjustment.value() == 0.0 {
                r.receiver[0].pan = 0;
            }
        });

        {
            let mut r = radio.lock().unwrap();
            r.receiver[0].init();
            if r.audio.local_output {
                let _result = r.audio.open_output();
            }
            if r.audio.local_input {
                let _result = r.audio.open_input();
            }
        }

        main_window.set_child(Some(&content));

        match device.protocol {
            1 => {
                let mut p1 = Protocol1::new(device);
                let radio_clone_for_spawn = Arc::clone(&radio);
                thread::spawn(move || {
                    p1.run(radio_clone_for_spawn);
                });
            },
            2 => {
                let mut p2 = Protocol2::new(device);
                let radio_clone_for_spawn = Arc::clone(&radio);
                thread::spawn(move || {
                    p2.run(radio_clone_for_spawn);
                });
            },
            _ => eprintln!("Invalid protocol"),
        }


        let r = radio.lock().unwrap();
        let update_interval = 1000.0 / r.receiver[0].fps;
        drop(r);
        let spectrum_display_for_timeout = spectrum_display.clone();
        let waterfall_display_for_timeout = waterfall_display.clone();
        let meter_display_for_timeout = meter_display.clone();
        let radio_clone_for_timeout = Arc::clone(&radio);
        //let band_info_for_timeout = band_info.clone();
        let pixbuf_for_timeout = pixbuf.clone();
        glib::timeout_add_local(Duration::from_millis(update_interval as u64), move || {
            let r = radio_clone_for_timeout.lock().unwrap();
            let zoom = r.receiver[0].zoom;
            let channel = r.receiver[0].channel;
            let subrx = r.receiver[0].subrx;
            let subrx_channel = r.receiver[0].subrx_channel;
            drop(r);

            let mut pixels = vec![0.0; (spectrum_display_for_timeout.width() * zoom) as usize];
            let mut flag: c_int = 0;
            unsafe {
                GetPixels(channel, 0, pixels.as_mut_ptr(), &mut flag);
            }

            if flag != 0 {
                let pixbuf_for_draw = pixbuf_for_timeout.clone();
                let waterfall_display_for_draw = waterfall_display_for_timeout.clone();
                let radio_clone_for_draw = Arc::clone(&radio_clone_for_timeout);
                spectrum_display_for_timeout.set_draw_func(move |_da, cr, width, height|{
                    {
                        draw_spectrum(cr, width, height, &radio_clone_for_draw, &pixels);
                        let pixbuf_for_waterfall = pixbuf_for_draw.clone();
                        update_waterfall(width, height, &radio_clone_for_draw, &pixbuf_for_waterfall, &pixels);
                    }

                    unsafe {
                        let mut r = radio_clone_for_draw.lock().unwrap();
                        r.s_meter_dbm = GetRXAMeter(channel,rxaMeterType_RXA_S_AV as i32);
                        if subrx {
                            r.subrx_s_meter_dbm = GetRXAMeter(subrx_channel,rxaMeterType_RXA_S_AV as i32);
                        }
                    }


                    let pixbuf_for_waterfall_draw = pixbuf_for_draw.clone();
                    waterfall_display_for_draw.set_draw_func(move |_da, cr, width, height| {
                        draw_waterfall(cr, width, height, &pixbuf_for_waterfall_draw);
                    });
                });

                spectrum_display_for_timeout.queue_draw();
                waterfall_display_for_timeout.queue_draw();
                meter_display_for_timeout.queue_draw();
            }

            Continue

        });

    }


    fn config_file_path(device: Device) -> PathBuf {
        let d = format!("{:02X}-{:02X}-{:02X}-{:02X}-{:02X}-{:02X}", device.mac[0], device.mac[1], device.mac[2], device.mac[3], device.mac[4], device.mac[5]);
        let app_name = env!("CARGO_PKG_NAME");
        let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        config_dir.join(app_name).join(d).join("radio.ron")
    }

    pub fn load(device: Device) -> Self {
        let path = Self::config_file_path(device);
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(s) => match ron::from_str(&s) {
                    Ok(radio) => {
                        println!("Successfully loaded radio data from {:?}", path);
                        radio
                    }
                    Err(_e) => {
                        Self::new(device)
                    }
                },
                Err(_e) => {
                    Self::new(device)
                }
            }
        } else {
            Self::new(device)
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

fn spectrum_waterfall_clicked(radio: &Arc<Mutex<Radio>>, fa: &Label, fb: &Label, x: f64, width: i32, button: u32, button_subrx: &ToggleButton) {
    let mut r = radio.lock().unwrap();
        
    let frequency_low = r.receiver[0].frequency_a - (r.receiver[0].sample_rate/2) as f32;
    let frequency_high = r.receiver[0].frequency_a + (r.receiver[0].sample_rate/2) as f32;
    let frequency_range = frequency_high - frequency_low;
                
    let display_frequency_range = frequency_range / r.receiver[0].zoom as f32;
    let display_frequency_offset = ((frequency_range - display_frequency_range) / 100.0) * r.receiver[0].pan as f32;  
    let display_frequency_low = frequency_low + display_frequency_offset;
    let display_frequency_high = frequency_high + display_frequency_offset;
    let display_hz_per_pixel = display_frequency_range as f32 / width as f32;


    let f1 = display_frequency_low + (x as f32 * display_hz_per_pixel);
    let f1 = (f1 as u32 / r.receiver[0].step as u32 * r.receiver[0].step as u32) as f32;
 
    if button == 3 && r.receiver[0].subrx {
        r.receiver[0].frequency_b = f1;
        r.receiver[0].set_subrx_frequency();
        let formatted_value = format_u32_with_separators(r.receiver[0].frequency_b as u32);
        fb.set_label(&formatted_value);
    } else if r.receiver[0].ctun {
        r.receiver[0].ctun_frequency = f1;
        r.receiver[0].set_ctun_frequency();
        let formatted_value = format_u32_with_separators(r.receiver[0].ctun_frequency as u32);
        fa.set_label(&formatted_value);
    } else {
        r.receiver[0].frequency_a = f1;
        if r.receiver[0].subrx {
            if r.receiver[0].frequency_b < frequency_low || r.receiver[0].frequency_b > frequency_high {
                r.receiver[0].subrx = false;
                button_subrx.set_active(r.receiver[0].subrx);
            } else {
                r.receiver[0].set_subrx_frequency();
            }
        }
        let formatted_value = format_u32_with_separators(r.receiver[0].frequency_a as u32);
        fa.set_label(&formatted_value);
    }
}

fn spectrum_waterfall_scroll(radio: &Arc<Mutex<Radio>>, f: &Label, dy: f64, button_subrx: &ToggleButton) {
    let mut r = radio.lock().unwrap();
    let frequency_low = r.receiver[0].frequency_a - (r.receiver[0].sample_rate/2) as f32;
    let frequency_high = r.receiver[0].frequency_a + (r.receiver[0].sample_rate/2) as f32;
    if r.receiver[0].ctun {
        r.receiver[0].ctun_frequency = r.receiver[0].ctun_frequency - (r.receiver[0].step * dy as f32);
        if r.receiver[0].ctun_frequency < frequency_low {
            r.receiver[0].ctun_frequency = frequency_low;
        } else if r.receiver[0].ctun_frequency > frequency_high {
            r.receiver[0].ctun_frequency = frequency_high;
        }
        let formatted_value = format_u32_with_separators(r.receiver[0].ctun_frequency as u32);
        f.set_label(&formatted_value);
        r.receiver[0].set_ctun_frequency();
    } else {
        r.receiver[0].frequency_a = r.receiver[0].frequency_a - (r.receiver[0].step * dy as f32);
        if r.receiver[0].subrx {
            if r.receiver[0].frequency_b < frequency_low || r.receiver[0].frequency_b > frequency_high {
                r.receiver[0].subrx = false;
                button_subrx.set_active(r.receiver[0].subrx);
            } else {
                r.receiver[0].set_subrx_frequency();
            }
        }
        let formatted_value = format_u32_with_separators(r.receiver[0].frequency_a as u32);
        f.set_label(&formatted_value);
    }
}

fn draw_meter(cr: &Context, dbm: f64, subrx: bool) {
    let x_offset = 5.0;
    let mut y_offset = 0.0;
    if subrx {
        y_offset = 25.0;
    }
    let db = 1.0; // size in pixels of each dbm

    if subrx {
        cr.set_source_rgb(1.0, 0.5, 0.0);
    } else {
        cr.set_source_rgb(0.0, 1.0, 0.0);
    }
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

fn draw_spectrum(cr: &Context, width: i32, height: i32, radio: &Arc<Mutex<Radio>>, pixels: &Vec<f32>) {
    cr.set_source_rgb(0.0, 0.0, 0.0);
    cr.paint().unwrap();

    let r = radio.lock().unwrap();   
    let b = r.receiver[0].band.to_usize();
    let dbm_per_line: f32 = height as f32/(r.band_info[b].spectrum_high-r.band_info[b].spectrum_low);

    cr.set_source_rgb(1.0, 1.0, 0.0);                   
    cr.set_line_width(1.0);
    cr.set_line_cap(LineCap::Round);
    cr.set_line_join(LineJoin::Round);

    let frequency_low = r.receiver[0].frequency_a - (r.receiver[0].sample_rate/2) as f32;
    let frequency_high = r.receiver[0].frequency_a + (r.receiver[0].sample_rate/2) as f32;
    let frequency_range = frequency_high - frequency_low;
    //let hz_per_pixel = frequency_range as f32 / pixels.len() as f32;

    let display_frequency_range = frequency_range / r.receiver[0].zoom as f32;
    let display_frequency_offset = ((frequency_range - display_frequency_range) / 100.0) * r.receiver[0].pan as f32;
    let display_frequency_low = frequency_low + display_frequency_offset;
    let display_frequency_high = frequency_high + display_frequency_offset;
    let display_hz_per_pixel = display_frequency_range as f32 / width as f32;

    cr.set_source_rgb(0.5, 0.5, 0.5);
    let mut step = 25000.0;  
    match r.receiver[0].sample_rate {       
         48000 => step = 50000.0,
         96000 => step = 10000.0,   
        192000 => step = 20000.0,
        384000 => match r.receiver[0].zoom {
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
        if i % r.receiver[0].spectrum_step as i32 == 0 {
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
    let spectrum_width = r.receiver[0].spectrum_width;
    let pan = ((pixels.len() as f32 - spectrum_width as f32) / 100.0) * r.receiver[0].pan as f32;
    cr.set_source_rgb(1.0, 1.0, 0.0);
    cr.move_to(0.0, height as f64);
    for i in 0..spectrum_width {
        let pixel = pixels[i as usize + pan as usize];
        let y = ((spectrum_high - pixel as f32) * dbm_per_line).floor();  
        cr.line_to(i as f64, y.into());
    }
    cr.line_to(width as f64, height as f64);

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
    cr.stroke().unwrap();


    let mut frequency = r.receiver[0].frequency_a;
    if r.receiver[0].ctun {
        frequency = r.receiver[0].ctun_frequency;
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
/*
        if r.receiver[0].mode == Modes::CWL.to_usize() || r.receiver[0].mode == Modes::CWU.to_usize() {
            let filter_left = ((frequency + r.receiver[0].filter_low - r.receiver[0].cw_sidetone) - display_frequency_low) / display_hz_per_pixel;
            let filter_right = ((frequency + r.receiver[0].filter_high + r.receiver[0].cw_sidetone) - display_frequency_low) / display_hz_per_pixel;
            cr.rectangle(filter_left.into(), 0.0, (filter_right-filter_left).into(), height.into());
        } else {
*/
            let filter_left = ((frequency + r.receiver[0].filter_low) - display_frequency_low) / display_hz_per_pixel;
            let filter_right = ((frequency + r.receiver[0].filter_high) - display_frequency_low) / display_hz_per_pixel;
            cr.rectangle(filter_left.into(), 0.0, (filter_right-filter_left).into(), height.into());
/*
        }
*/
        let _ = cr.fill();
    }

    if r.receiver[0].subrx {
        frequency = r.receiver[0].frequency_b;
        if display_frequency_low < frequency && display_frequency_high > frequency {
            // draw the center line frequency marker
            let x = (frequency - display_frequency_low) / display_hz_per_pixel;
            cr.set_source_rgb(1.0, 0.64, 0.0); // orange
            cr.set_line_width(1.0);
            cr.move_to(x.into(), 0.0);
            cr.line_to(x.into(), height.into());
            cr.stroke().unwrap();

            // draw the filter
            cr.set_source_rgba (0.5, 0.5, 0.5, 0.50);
            if r.receiver[0].mode == Modes::CWL.to_usize() || r.receiver[0].mode == Modes::CWU.to_usize() {
                let filter_left = ((frequency + r.receiver[0].filter_low - r.receiver[0].cw_sidetone) - display_frequency_low) / display_hz_per_pixel;
                let filter_right = ((frequency + r.receiver[0].filter_high + r.receiver[0].cw_sidetone) - display_frequency_low) / display_hz_per_pixel;
                cr.rectangle(filter_left.into(), 0.0, (filter_right-filter_left).into(), height.into());
            } else {
                let filter_left = ((frequency + r.receiver[0].filter_low) - display_frequency_low) / display_hz_per_pixel;
                let filter_right = ((frequency + r.receiver[0].filter_high) - display_frequency_low) / display_hz_per_pixel;
                cr.rectangle(filter_left.into(), 0.0, (filter_right-filter_left).into(), height.into());
            }
            let _ = cr.fill();
        }
    }
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
    let r = radio.lock().unwrap();
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
                let spectrum_width = r.receiver[0].spectrum_width;
                let pan = ((new_pixels.len() as f32 - spectrum_width as f32) / 100.0) * r.receiver[0].pan as f32;

                for x in 0..spectrum_width {
                    let b = r.receiver[0].band.to_usize();
                    let mut value: f32 = new_pixels[x as usize + pan as usize] as f32;
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
