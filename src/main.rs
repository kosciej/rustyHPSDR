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

use glib::ControlFlow::Continue;
use glib::timeout_add_local;
use gtk::prelude::*;
use gtk::{Adjustment, Application, ApplicationWindow, Builder, Button, DrawingArea, DropDown, Grid, Label, Scale, ToggleButton, Window};
use gtk::{EventController, EventControllerMotion, EventControllerScroll, EventControllerScrollFlags, GestureClick};
use gtk::gdk::Cursor;
use gtk::glib::Propagation;

use std::cell::{Cell, RefCell};
use std::env;
use std::ffi::{CString, OsString};
use std::fs;
use std::os::raw::c_char;
use std::os::unix::ffi::OsStringExt;
use std::path::{Path, PathBuf};
use std::process;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use rustyHPSDR::agc::*;
use rustyHPSDR::bands::*;
use rustyHPSDR::modes::*;
use rustyHPSDR::filters::*;
use rustyHPSDR::discovery::create_discovery_dialog;
use rustyHPSDR::discovery::device_name;
use rustyHPSDR::radio::Radio;
use rustyHPSDR::radio::RadioMutex;
use rustyHPSDR::configure::*;
use rustyHPSDR::protocol1::Protocol1;
use rustyHPSDR::protocol2::Protocol2;
use rustyHPSDR::spectrum::*;
use rustyHPSDR::waterfall::*;
use rustyHPSDR::meter::*;
use rustyHPSDR::util::*;
use rustyHPSDR::wdsp::*;

struct AppWidgets {
    pub main_window: ApplicationWindow,
    pub configure_button: Button,
    pub vfo_a_frequency: Label,
    pub vfo_b_frequency: Label,
    pub a_to_b_button: Button,
    pub b_to_a_button: Button,
    pub a_swap_b_button: Button,
    pub split_button: Button,
    pub subrx_button: ToggleButton,
    pub ctun_button: ToggleButton,
    pub step_dropdown: DropDown,
    pub meter_display: DrawingArea,
    pub spectrum_display: DrawingArea,
    pub waterfall_display: DrawingArea,
    pub band_grid: Grid,
    pub mode_grid: Grid,
    pub filter_grid: Grid,
    pub zoom_adjustment: Adjustment,
    pub pan_adjustment: Adjustment,
    pub nr_button: Button,
    pub nb_button: Button,
    pub anf_button: ToggleButton,
    pub snb_button: ToggleButton,
    pub mox_button: ToggleButton,
    pub tun_button: ToggleButton,
    pub afgain_adjustment: Adjustment,
    pub agc_dropdown: DropDown,
    pub agcgain_adjustment: Adjustment,
    pub adcattn_adjustment: Adjustment,
    pub micgain_adjustment: Adjustment,
    pub drive_adjustment: Adjustment,
}

impl AppWidgets {
    fn from_builder(builder: &Builder) -> Self {
        let main_window: ApplicationWindow = builder
            .object("main_window")
            .expect("Could not get object `main_window` from builder.");

        let configure_button: Button = builder
            .object("configure_button")
            .expect("Could not get configure_button from builder");

        let vfo_a_frequency: Label = builder
            .object("vfo_a_frequency")
            .expect("Could not get vfo_a_frequency from builder");

        let vfo_b_frequency: Label = builder
            .object("vfo_b_frequency")
            .expect("Could not get vfo_b_frequency from builder");

        let a_to_b_button: Button = builder
            .object("a_to_b_button")
            .expect("Could not get a_to_b_button from builder");

        let b_to_a_button: Button = builder
            .object("b_to_a_button")
            .expect("Could not get b_to_a_button from builder");

        let a_swap_b_button: Button = builder
            .object("a_swap_b_button")
            .expect("Could not get a_swap_b_button from builder");

        let split_button: Button = builder
            .object("split_button")
            .expect("Could not get split_button from builder");

        let subrx_button: ToggleButton = builder
            .object("subrx_button")
            .expect("Could not get subrx_button from builder");

        let ctun_button: ToggleButton = builder
            .object("ctun_button")
            .expect("Could not get ctun_button from builder");

        let step_dropdown = builder
            .object("step_dropdown")
            .expect("Could not get step_dropdown from builder");

        let meter_display: DrawingArea = builder
            .object("meter_display")
            .expect("Could not get meter_display from builder");

        let spectrum_display: DrawingArea = builder
            .object("spectrum_display")
            .expect("Could not get spectrum_display from builder");

        let waterfall_display: DrawingArea = builder
            .object("waterfall_display")
            .expect("Could not get waterfall_display from builder");

        let band_grid: Grid = builder
            .object("band_grid")
            .expect("Could not get band_grid from builder");

        let mode_grid: Grid = builder
            .object("mode_grid")
            .expect("Could not get mode_grid from builder");

        let filter_grid: Grid = builder
            .object("filter_grid")
            .expect("Could not get filter_grid from builder");

        let zoom_adjustment: Adjustment = builder
            .object("zoom_adjustment")
            .expect("Could not get zoom_adjustment from builder");

        let pan_adjustment: Adjustment = builder
            .object("pan_adjustment")
            .expect("Could not get pan_adjustment from builder");

        let nr_button: Button = builder
            .object("nr_button")
            .expect("Could not get nr_button from builder");

        let nb_button: Button = builder
            .object("nb_button")
            .expect("Could not get nb_button from builder");

        let anf_button: ToggleButton = builder
            .object("anf_button")
            .expect("Could not get anf_button from builder");

        let snb_button: ToggleButton = builder
            .object("snb_button")
            .expect("Could not get snb_button from builder");

        let mox_button: ToggleButton = builder
            .object("mox_button")
            .expect("Could not get mox_button from builder");

        let tun_button: ToggleButton = builder
            .object("tun_button")
            .expect("Could not get tun_button from builder");

        let afgain_adjustment: Adjustment = builder
            .object("afgain_adjustment")
            .expect("Could not get afgain_adjustment from builder");

        let agc_dropdown: DropDown = builder
            .object("agc_dropdown")
            .expect("Could not get agc_dropdown from builder");

        let agcgain_adjustment: Adjustment = builder
            .object("agcgain_adjustment")
            .expect("Could not get agcgain_adjustment from builder");

        let adcattn_adjustment: Adjustment = builder
            .object("attenuation_adjustment")
            .expect("Could not get attenuation_adjustment from builder");

        let micgain_adjustment: Adjustment = builder
            .object("micgain_adjustment")
            .expect("Could not get micgain_adjustment from builder");

        let drive_adjustment: Adjustment = builder
            .object("drive_adjustment")
            .expect("Could not get drive_adjustment from builder");

        AppWidgets {
            main_window,
            configure_button,
            vfo_a_frequency,
            vfo_b_frequency,
            a_to_b_button,
            b_to_a_button,
            a_swap_b_button,
            split_button,
            subrx_button,
            ctun_button,
            step_dropdown,
            meter_display,
            spectrum_display,
            waterfall_display,
            band_grid,
            mode_grid,
            filter_grid,
            zoom_adjustment,
            pan_adjustment,
            nr_button,
            nb_button,
            anf_button,
            snb_button,
            mox_button,
            tun_button,
            afgain_adjustment,
            agc_dropdown,
            agcgain_adjustment,
            adcattn_adjustment,
            micgain_adjustment,
            drive_adjustment,
        }
    }
}

fn main() {
    let id = format!("org.g0orx.rustyHPSDR.pid{}", process::id());
    let application = Application::builder()
        .application_id(id)
        .build();
    application.connect_activate(build_ui);
    application.run();
}

fn build_ui(app: &Application) {

    // check wisdom file exists - if not create it
    let home_dir: PathBuf = match env::home_dir() {
        Some(path) => path,
        None => {
            eprintln!("Error: Could not determine home directory.");
            return;
        }
    };

    let my_dir = home_dir.join(".config").join("rustyHPSDR").join("");
    if !my_dir.is_dir() {
        match fs::create_dir_all(&my_dir) {
            Ok(_) => {
            }
            Err(e) => {
                let error_message = format!("Failed to create directory {:?}: {}", my_dir, e);
                eprintln!("{}", error_message);
            }
        }
    }

    let os_string = my_dir.clone().into_os_string();
    let c_string = match CString::new(os_string.into_vec()) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error converting path to C string (contains null bytes?): {}", e);
            return;
        }
    };
    let c_path_ptr: *const c_char = c_string.as_ptr();
    unsafe {
        WDSPwisdom(c_path_ptr);
    }

    let ui_css = include_str!("ui/ui.css");
    let provider = gtk::CssProvider::new();
    provider.load_from_data(ui_css);
    gtk::StyleContext::add_provider_for_display(
        &gtk::gdk::Display::default().unwrap(),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let ui_xml = include_str!("ui/ui.xml");
    let builder = Builder::from_string(ui_xml);

    let app_widgets = AppWidgets::from_builder(&builder);
    let rc_app_widgets = Rc::new(RefCell::new(app_widgets));

    let rc_app_widgets_clone = rc_app_widgets.clone();
    let app_widgets = rc_app_widgets_clone.borrow();
    app_widgets.main_window.set_application(Some(app));

    let mut spectrum = Spectrum::new(1024,168);
    let rc_spectrum = Rc::new(RefCell::new(spectrum));
    let mut waterfall = Waterfall::new(1024,168);
    let rc_waterfall = Rc::new(RefCell::new(waterfall));
    let mut meter = Meter::new(256,72);
    let rc_meter = Rc::new(RefCell::new(meter));

    let discovery_data = Rc::new(RefCell::new(Vec::new()));
    let selected_index: Rc<RefCell<Option<i32>>> = Rc::new(RefCell::new(None));
    let selected_index_for_discovery_dialog = selected_index.clone();
    let discovery_data_clone = Rc::clone(&discovery_data);

    let rc_app_widgets_clone = rc_app_widgets.clone();
    let app_widgets = rc_app_widgets_clone.borrow();
    let discovery_dialog = create_discovery_dialog(&app_widgets.main_window.clone(), discovery_data_clone, selected_index_for_discovery_dialog);


    let selected_index_clone = selected_index.clone();
    let discovery_data_clone = Rc::clone(&discovery_data);
    let app_clone = app.clone();
    let builder_clone = builder.clone();
    let rc_spectrum_clone = rc_spectrum.clone();
    let rc_waterfall_clone = rc_waterfall.clone();
    let rc_meter_clone = rc_meter.clone();
    let rc_app_widgets_clone = rc_app_widgets.clone();
    discovery_dialog.connect_close_request(move |_| {
        let app_widgets = rc_app_widgets_clone.borrow();
        let index = *selected_index_clone.borrow();
        match index {
            Some(i) => {
                if i >= 0 {
                    let device = discovery_data_clone.borrow()[(i-1) as usize];

                    let title = format!("rustyHPSDR: {} {:?} Protocol {}", device_name(device), device.address, device.protocol);
                    app_widgets.main_window.set_title(Some(&title));

                    let radio_mutex = RadioMutex::new(Arc::new(Mutex::new(Radio::load(device, app_widgets.spectrum_display.width()))));

                    let mut rc_spectrum_clone2 = rc_spectrum_clone.clone();
                    let radio_mutex_clone = radio_mutex.clone();
                    app_widgets.spectrum_display.connect_resize(move |_, width, height| { 
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        r.receiver[0].spectrum_width = width;
                        r.receiver[0].init_analyzer(r.receiver[0].channel);
                        let mut spectrum = rc_spectrum_clone2.borrow_mut();
                        spectrum.resize(width, height);
                    });

                    let mut rc_waterfall_clone2 = rc_waterfall_clone.clone();
                    let radio_mutex_clone = radio_mutex.clone();
                    app_widgets.waterfall_display.connect_resize(move |_, width, height| {
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        r.receiver[0].waterfall_width = width;
                        let mut waterfall = rc_waterfall_clone2.borrow_mut();
                        waterfall.resize(width, height);
                    });

                    // setup the ui state
                    {
                        let mut r = radio_mutex.radio.lock().unwrap();

                        if r.receiver[0].ctun {
                            let formatted_value = format_u32_with_separators(r.receiver[0].ctun_frequency as u32);
                            app_widgets.vfo_a_frequency.set_label(&formatted_value);
                        } else {
                            let formatted_value = format_u32_with_separators(r.receiver[0].frequency_a as u32);
                            app_widgets.vfo_a_frequency.set_label(&formatted_value);
                        }

                        let formatted_value = format_u32_with_separators(r.receiver[0].frequency_b as u32);
                        app_widgets.vfo_b_frequency.set_label(&formatted_value);

                        app_widgets.ctun_button.set_active(r.receiver[0].ctun);
                        if r.receiver[0].ctun {
                            app_widgets.ctun_button.add_css_class("active-button");
                        } else {
                            app_widgets.ctun_button.add_css_class("inactive-button");
                        }

                        app_widgets.subrx_button.set_active(r.receiver[0].subrx);
                        if r.receiver[0].subrx{
                            app_widgets.subrx_button.add_css_class("active-button");
                        } else {
                            app_widgets.subrx_button.add_css_class("inactive-button");
                        }

                        app_widgets.afgain_adjustment.set_value((r.receiver[0].afgain * 100.0).into());
                        app_widgets.agc_dropdown.set_selected(r.receiver[0].agc as u32);
                        app_widgets.agcgain_adjustment.set_value(r.receiver[0].agcgain.into());
                        app_widgets.micgain_adjustment.set_value(r.transmitter.micgain.into());
                        app_widgets.micgain_adjustment.set_value(r.transmitter.micgain.into());
                        app_widgets.drive_adjustment.set_value(r.transmitter.drive.into());

                        let mut rc_spectrum_clone2 = rc_spectrum_clone.clone();
                        let mut spectrum = rc_spectrum_clone2.borrow_mut();
                        spectrum.resize(app_widgets.spectrum_display.width(), app_widgets.spectrum_display.height());
                        r.receiver[0].spectrum_width = app_widgets.spectrum_display.width();
                        r.receiver[0].init();
                        r.receiver[0].init_analyzer(r.receiver[0].channel);

                        let mut rc_waterfall_clone2 = rc_waterfall_clone.clone();
                        let mut waterfall = rc_waterfall_clone2.borrow_mut();
                        waterfall.resize(app_widgets.waterfall_display.width(), app_widgets.waterfall_display.height());
                        r.receiver[0].spectrum_width = app_widgets.spectrum_display.width();

                        app_widgets.step_dropdown.set_selected(r.receiver[0].step_index as u32);

                        app_widgets.zoom_adjustment.set_value(r.receiver[0].zoom.into());
                        app_widgets.pan_adjustment.set_value(r.receiver[0].pan.into());
                       
                    }

                    // handle ui events
                    let radio_mutex_clone = radio_mutex.clone();
                    let rc_app_widgets_clone_clone = rc_app_widgets_clone.clone();
                    app_widgets.configure_button.connect_clicked(move |_| {
                        let app_widgets = rc_app_widgets_clone_clone.borrow();
                        let configure_dialog = create_configure_dialog(&app_widgets.main_window, &radio_mutex_clone);
                        app_widgets.configure_button.set_sensitive(false);
                        configure_dialog.present();
                        let rc_app_widgets = rc_app_widgets_clone_clone.clone();
                        configure_dialog.connect_close_request(move |_| {
                            let app_widgets = rc_app_widgets.borrow();
                            app_widgets.configure_button.set_sensitive(true);
                            Propagation::Proceed
                        });
                    });                         

                    let scroll_controller_a = EventControllerScroll::new(
                        EventControllerScrollFlags::VERTICAL
                    );
                    let radio_mutex_clone = radio_mutex.clone();
                    let rc_app_widgets_clone_clone = rc_app_widgets_clone.clone();
                    scroll_controller_a.connect_scroll(move |controller, dx, dy| {
                        spectrum_waterfall_scroll(&radio_mutex_clone, &rc_app_widgets_clone_clone, dy);
                        Propagation::Proceed
                    });
                    app_widgets.vfo_a_frequency.add_controller(scroll_controller_a);

                    let scroll_controller_b = EventControllerScroll::new(
                        EventControllerScrollFlags::VERTICAL
                    );
                    let radio_mutex_clone = radio_mutex.clone();
                    let rc_app_widgets_clone_clone = rc_app_widgets_clone.clone();
                    scroll_controller_b.connect_scroll(move |_controller, _dx, dy| {
                        let app_widgets = rc_app_widgets_clone_clone.borrow();
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
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
                        app_widgets.vfo_b_frequency.set_label(&formatted_value);
                        Propagation::Proceed
                    });
                    app_widgets.vfo_b_frequency.add_controller(scroll_controller_b);


                    let radio_mutex_clone = radio_mutex.clone();
                    let rc_app_widgets_clone_clone = rc_app_widgets_clone.clone();
                    app_widgets.a_to_b_button.connect_clicked(move |_| {
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        let app_widgets = rc_app_widgets_clone_clone.borrow();
                        if r.receiver[0].ctun {
                            r.receiver[0].frequency_b = r.receiver[0].ctun_frequency; 
                        } else {
                            r.receiver[0].frequency_b = r.receiver[0].frequency_a; 
                        }
                        let formatted_value = format_u32_with_separators(r.receiver[0].frequency_b as u32);
                        app_widgets.vfo_b_frequency.set_label(&formatted_value);
                    });                         
                    
                    let radio_mutex_clone = radio_mutex.clone();
                    let rc_app_widgets_clone_clone = rc_app_widgets_clone.clone();
                    app_widgets.b_to_a_button.connect_clicked(move |_| {
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        let app_widgets = rc_app_widgets_clone_clone.borrow();
                        if r.receiver[0].ctun {
                            r.receiver[0].ctun_frequency = r.receiver[0].frequency_b;
                            r.receiver[0].set_ctun_frequency();
                        } else {
                            r.receiver[0].frequency_a = r.receiver[0].frequency_b;
                        }
                        let formatted_value = format_u32_with_separators(r.receiver[0].frequency_a as u32);
                        app_widgets.vfo_a_frequency.set_label(&formatted_value);
                    });

                    let radio_mutex_clone = radio_mutex.clone();
                    let rc_app_widgets_clone_clone = rc_app_widgets_clone.clone();
                    app_widgets.a_swap_b_button.connect_clicked(move |_| {
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        let temp_frequency = r.receiver[0].frequency_b;
                        let app_widgets = rc_app_widgets_clone_clone.borrow();
                        if r.receiver[0].ctun {
                            r.receiver[0].frequency_b = r.receiver[0].ctun_frequency;
                            r.receiver[0].ctun_frequency = temp_frequency;
                            let formatted_value = format_u32_with_separators(r.receiver[0].ctun_frequency as u32);
                            app_widgets.vfo_a_frequency.set_label(&formatted_value);
                            r.receiver[0].set_ctun_frequency();
                        } else {
                            r.receiver[0].frequency_b = r.receiver[0].frequency_a;
                            r.receiver[0].frequency_a = temp_frequency;
                            let formatted_value = format_u32_with_separators(r.receiver[0].frequency_a as u32);
                            app_widgets.vfo_a_frequency.set_label(&formatted_value);
                        }
                        let formatted_value = format_u32_with_separators(r.receiver[0].frequency_b as u32);
                        app_widgets.vfo_b_frequency.set_label(&formatted_value);
                    });

                    let radio_mutex_clone = radio_mutex.clone();
                    let rc_app_widgets_clone_clone = rc_app_widgets_clone.clone();
                    app_widgets.ctun_button.connect_clicked(move |button| {
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        let app_widgets = rc_app_widgets_clone_clone.borrow();
                        if r.receiver[0].ctun {
                            r.receiver[0].ctun_frequency = 0.0;
                            r.receiver[0].ctun = false;
                            button.remove_css_class("active-button");
                            button.add_css_class("inactive-button");
                            r.receiver[0].set_ctun(false);
                            let formatted_value = format_u32_with_separators(r.receiver[0].frequency_a as u32);
                            app_widgets.vfo_a_frequency.set_label(&formatted_value);
                        } else {
                            r.receiver[0].ctun_frequency = r.receiver[0].frequency_a;
                            r.receiver[0].ctun = true;
                            button.remove_css_class("inactive-button");
                            button.add_css_class("active-button");
                            r.receiver[0].set_ctun(true);
                        }
                    });

                    let radio_mutex_clone = radio_mutex.clone();
                    let rc_app_widgets_clone_clone = rc_app_widgets_clone.clone();
                    app_widgets.subrx_button.connect_clicked(move |button| {
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        let app_widgets = rc_app_widgets_clone_clone.borrow();
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
                            app_widgets.vfo_b_frequency.set_label(&formatted_value);
                        }
                    });
                        
                    let radio_mutex_clone = radio_mutex.clone();
                    app_widgets.step_dropdown.connect_selected_notify(move |step_dropdown| {
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        let index = step_dropdown.selected();
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

                    let middle_button_pressed = Rc::new(RefCell::new(false));
                    let spectrum_click_gesture = Rc::new(GestureClick::new());
                    spectrum_click_gesture.set_button(0); // all buttons
                    let spectrum_click_gesture_clone = spectrum_click_gesture.clone();
                    let radio_mutex_clone = radio_mutex.clone();
                    let rc_app_widgets_clone_clone = rc_app_widgets_clone.clone();
                    let press_state = middle_button_pressed.clone();
                    spectrum_click_gesture_clone.connect_pressed(move |gesture, _, x, _y| {
                        let da = gesture.widget().unwrap();
                        let width = da.allocated_width();
                        if gesture.current_button() == 2 { // middle button
                            *press_state.borrow_mut() = true;
                        } else {
                            spectrum_waterfall_clicked(&radio_mutex_clone, &rc_app_widgets_clone_clone, x, width, gesture.current_button());
                        }
                    });
                    let press_state = middle_button_pressed.clone();
                    spectrum_click_gesture_clone.connect_released(move |gesture, _, x, _y| {
                        if gesture.current_button() == 2 { // middle button
                            *press_state.borrow_mut() = false;
                        }
                    });

                    app_widgets.spectrum_display.add_controller(<GestureClick as Clone>::clone(&spectrum_click_gesture).upcast::<EventController>());

                    let last_spectrum_x = Rc::new(Cell::new(0.0));
                    let last_spectrum_y = Rc::new(Cell::new(0.0));

                    let cursor_nsresize = Cursor::from_name("ns-resize", None);
                    let cursor_uparrow = Cursor::from_name("up-arrow", None);
                    let cursor_downarrow = Cursor::from_name("down-arrow", None);
                    let motion_event_controller_spectrum = EventControllerMotion::new();
                    app_widgets.spectrum_display.add_controller(motion_event_controller_spectrum.clone());
                    let last_spectrum_x_clone = last_spectrum_x.clone();
                    let last_spectrum_y_clone = last_spectrum_y.clone();
                    let rc_app_widgets_clone_clone = rc_app_widgets_clone.clone();
                    motion_event_controller_spectrum.connect_motion(move |_controller, x, y| {
                        last_spectrum_x_clone.set(x);
                        last_spectrum_y_clone.set(y);
                        let app_widgets = rc_app_widgets_clone_clone.borrow();
                        if x < 40.0 {
                            let height = app_widgets.spectrum_display.height();
                            let top = height / 4;
                            let bottom = height - top;
                            if y < top.into() {
                                app_widgets.spectrum_display.set_cursor(cursor_uparrow.as_ref());
                            } else if y > bottom.into() {
                                app_widgets.spectrum_display.set_cursor(cursor_downarrow.as_ref());
                            } else {
                                app_widgets.spectrum_display.set_cursor(cursor_nsresize.as_ref());
                            }
                        } else {
                            app_widgets.spectrum_display.set_cursor(None); // default
                        }
                    });

                    let scroll_controller_spectrum = EventControllerScroll::new(
                        EventControllerScrollFlags::VERTICAL | EventControllerScrollFlags::KINETIC
                    );
                    let radio_mutex_clone = radio_mutex.clone();
                    let rc_app_widgets_clone_clone = rc_app_widgets_clone.clone();
                    let last_spectrum_x_clone = last_spectrum_x.clone();
                    let last_spectrum_y_clone = last_spectrum_y.clone();
                    let middle_button_state = middle_button_pressed.clone();
                    scroll_controller_spectrum.connect_scroll(move |controller, _dx, dy| {
                        let app_widgets = rc_app_widgets_clone_clone.borrow();
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        let subrx = r.receiver[0].subrx;
                        drop(r);
                        let mut increment = 1.0;
                        if dy > 0.0 {
                            increment = -1.0;
                        }
                        let height = app_widgets.spectrum_display.height();
                        let top = height / 4;
                        let bottom = height - top;

                        if *middle_button_state.borrow() && subrx {
                            let mut r = radio_mutex_clone.radio.lock().unwrap();
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
                            app_widgets.vfo_b_frequency.set_label(&formatted_value);
                        } else if last_spectrum_x_clone.get() < 40.0 {
                            let mut r = radio_mutex_clone.radio.lock().unwrap();
                            if r.is_transmitting() {
                                if last_spectrum_y_clone.get() < top.into() {
                                    r.transmitter.spectrum_high = r.transmitter.spectrum_high + increment;
                                } else if last_spectrum_y_clone.get() > bottom.into() {
                                    r.transmitter.spectrum_low = r.transmitter.spectrum_low + increment;
                                } else {
                                    r.transmitter.spectrum_high = r.transmitter.spectrum_high + increment;
                                    r.transmitter.spectrum_low = r.transmitter.spectrum_low + increment;
                                }
                            } else {
                                let b = r.receiver[0].band.to_usize();
                                if last_spectrum_y_clone.get() < top.into() {
                                    r.band_info[b].spectrum_high = r.band_info[b].spectrum_high + increment;
                                } else if last_spectrum_y_clone.get() > bottom.into() {
                                    r.band_info[b].spectrum_low = r.band_info[b].spectrum_low + increment;
                                } else {
                                    r.band_info[b].spectrum_low = r.band_info[b].spectrum_low + increment;
                                    r.band_info[b].spectrum_high = r.band_info[b].spectrum_high + increment;
                                }
                            }
                        } else {
                            spectrum_waterfall_scroll(&radio_mutex_clone, &rc_app_widgets_clone_clone, dy);
                        }
                        Propagation::Proceed
                    });
                    app_widgets.spectrum_display.add_controller(scroll_controller_spectrum.clone());

                    let scroll_controller_waterfall = EventControllerScroll::new(
                        EventControllerScrollFlags::VERTICAL | EventControllerScrollFlags::KINETIC
                    );
                    let radio_mutex_clone = radio_mutex.clone();
                    let rc_app_widgets_clone_clone = rc_app_widgets_clone.clone();
                    scroll_controller_waterfall.connect_scroll(move |controller, _dx, dy| {
                        spectrum_waterfall_scroll(&radio_mutex_clone, &rc_app_widgets_clone_clone, dy);
                        Propagation::Proceed
                    });
                    app_widgets.waterfall_display.add_controller(scroll_controller_waterfall.clone());

                    let waterfall_click_gesture = Rc::new(GestureClick::new());
                    waterfall_click_gesture.set_button(0); // all buttons
                    let waterfall_click_gesture_clone = waterfall_click_gesture.clone();
                    let radio_mutex_clone = radio_mutex.clone();
                    let rc_app_widgets_clone_clone = rc_app_widgets_clone.clone();
                    waterfall_click_gesture_clone.connect_pressed(move |gesture, _, x, _y| {
                        let da = gesture.widget().unwrap();
                        let width = da.allocated_width();
                        spectrum_waterfall_clicked(&radio_mutex_clone, &rc_app_widgets_clone_clone, x, width, gesture.current_button());
                    });
                    app_widgets.waterfall_display.add_controller(<GestureClick as Clone>::clone(&waterfall_click_gesture).upcast::<EventController>());


                    let radio_mutex_clone = radio_mutex.clone();
                    let rc_app_widgets_clone_clone = rc_app_widgets_clone.clone();
                    app_widgets.zoom_adjustment.connect_value_changed(move |adjustment| {
                        let app_widgets = rc_app_widgets_clone_clone.borrow();
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        r.receiver[0].zoom = adjustment.value() as i32;
                        r.receiver[0].init_analyzer(r.receiver[0].channel);
                        let mut p = 0.0;
                        if adjustment.value() == 1.0 {
                            r.receiver[0].pan = p as i32;
                        } else {
                            // try to keep the current frequency in the zoomed area
                            let frequency_low = r.receiver[0].frequency_a - (r.receiver[0].sample_rate/2) as f32;
                            let frequency_high = r.receiver[0].frequency_a + (r.receiver[0].sample_rate/2) as f32;
                            let frequency_range = frequency_high - frequency_low;
                            let width = r.receiver[0].spectrum_width * r.receiver[0].zoom;
                            let mut f = r.receiver[0].frequency_a;
                            let hz_per_pixel = frequency_range as f32 / width as f32;
                            if r.receiver[0].ctun {
                                f = r.receiver[0].ctun_frequency;
                            }
                            p = (f - frequency_low) / hz_per_pixel;
                            p = (p / width as f32) * 100.0;
                            r.receiver[0].pan = p as i32;
                        }
                        drop(r);
                        app_widgets.pan_adjustment.set_value(p as f64);
                    });

                    let radio_mutex_clone = radio_mutex.clone();
                    app_widgets.pan_adjustment.connect_value_changed(move |adjustment| {
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        if r.receiver[0].zoom > 1 {
                            r.receiver[0].pan = adjustment.value() as i32;
                        } else {
                            r.receiver[0].pan = 0;
                            adjustment.set_value(r.receiver[0].pan.into());
                        }
                    });

                    let mut band_grid = BandGrid::new(&radio_mutex.clone(), &builder);
                    let mut mode_grid = ModeGrid::new(&builder);
                    let mut filter_grid = FilterGrid::new(&builder);

                    let mut r = radio_mutex.radio.lock().unwrap();
                    let band = r.receiver[0].band.to_usize();
                    let mode = r.receiver[0].mode;
                    let filter = r.receiver[0].filter;
                    let low = r.receiver[0].filter_low;
                    let high = r.receiver[0].filter_high;
                    drop(r);

                    filter_grid.set_active_values(low, high);

                    let radio_mutex_clone = radio_mutex.clone();
                    let rc_app_widgets_clone_clone = rc_app_widgets_clone.clone();
                    let mut band_grid_clone = band_grid.clone();
                    let filter_grid_clone = filter_grid.clone();
                    band_grid.set_callback(move|index| {
                        let app_widgets = rc_app_widgets_clone_clone.borrow_mut();
                        // save current band info
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
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
                            app_widgets.subrx_button.set_active(r.receiver[0].subrx);
                        }

                        filter_grid_clone.update_filter_buttons(r.band_info[index].mode.to_usize());
                        filter_grid_clone.set_active_index(r.band_info[index].filter.to_usize());
                        r.receiver[0].mode = r.band_info[index].mode.to_usize();
                        let (low, high) = filter_grid_clone.get_filter_values(r.band_info[index].mode.to_usize(), r.band_info[index].filter.to_usize());
                        filter_grid_clone.set_active_values(low, high);
                        r.receiver[0].filter_low = low;
                        r.receiver[0].filter_high = high;
                        if r.receiver[0].mode == Modes::CWL.to_usize() {
                            r.receiver[0].filter_low = -r.receiver[0].cw_pitch - low;
                            r.receiver[0].filter_high = -r.receiver[0].cw_pitch + high;
                        } else if r.receiver[0].mode == Modes::CWU.to_usize() {
                            r.receiver[0].filter_low = r.receiver[0].cw_pitch - low;
                            r.receiver[0].filter_high = r.receiver[0].cw_pitch + high;
                        }
                        r.receiver[0].set_mode();

                        r.transmitter.filter_low = low;
                        r.transmitter.filter_high = high;
                        r.transmitter.mode = r.band_info[index].mode.to_usize();
                        r.transmitter.set_mode();
                        r.transmitter.set_filter();

                        let formatted_value = format_u32_with_separators(r.receiver[0].frequency_a as u32);
                        app_widgets.vfo_a_frequency.set_label(&formatted_value);

                    }, band);


                    let radio_mutex_clone = radio_mutex.clone();
                    let rc_app_widgets_clone_clone = rc_app_widgets_clone.clone();
                    let filter_grid_clone = filter_grid.clone();
                    mode_grid.set_callback(move|index| {
                        let app_widgets = rc_app_widgets_clone_clone.borrow_mut();
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        r.receiver[0].mode = index; 
                        filter_grid_clone.update_filter_buttons(index);

                        let (low, high) = filter_grid_clone.get_filter_values(index, r.receiver[0].filter);
                        filter_grid_clone.set_active_values(low, high);
                        r.receiver[0].filter_low = low;
                        r.receiver[0].filter_high = high;

                        if r.receiver[0].mode == Modes::CWL.to_usize() {
                            r.receiver[0].filter_low = -r.receiver[0].cw_pitch - low;
                            r.receiver[0].filter_high = -r.receiver[0].cw_pitch + high;
                        } else if r.receiver[0].mode == Modes::CWU.to_usize() {
                            r.receiver[0].filter_low = r.receiver[0].cw_pitch - low;
                            r.receiver[0].filter_high = r.receiver[0].cw_pitch + high;
                        }
                        r.receiver[0].set_mode();

                        r.transmitter.mode = index;
                        r.transmitter.set_mode();
                        r.transmitter.filter_low = low;
                        r.transmitter.filter_high = high;
                        r.transmitter.set_filter();
                    }, mode);

                    let radio_mutex_clone = radio_mutex.clone();
                    let filter_grid_clone = filter_grid.clone();
                    filter_grid.set_callback(move|index| {
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        r.receiver[0].filter = index;
                        let (low, high) = filter_grid_clone.get_filter_values(r.receiver[0].mode, r.receiver[0].filter);
                        filter_grid_clone.set_active_values(low, high);
                        r.receiver[0].filter_low = low;
                        r.receiver[0].filter_high = high;

                        if r.receiver[0].mode == Modes::CWL.to_usize() {
                            r.receiver[0].filter_low = -r.receiver[0].cw_pitch - low;
                            r.receiver[0].filter_high = -r.receiver[0].cw_pitch + high;
                        } else if r.receiver[0].mode == Modes::CWU.to_usize() {
                            r.receiver[0].filter_low = r.receiver[0].cw_pitch - low;
                            r.receiver[0].filter_high = r.receiver[0].cw_pitch + high;
                        }
                        r.receiver[0].set_filter();

                        r.transmitter.filter_low = low;
                        r.transmitter.filter_high = high;
                        r.transmitter.set_filter();
                    }, filter);

                    let radio_mutex_clone = radio_mutex.clone();
                    app_widgets.nr_button.clone().connect_clicked(move |button| {
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        if !(r.receiver[0].nr | r.receiver[0].nr2) {
                            r.receiver[0].nr = true;
                            button.remove_css_class("inactive-button");
                            button.add_css_class("active-button");
                            r.receiver[0].set_nr(); // turn on
                        } else if r.receiver[0].nr {
                            r.receiver[0].nr = false;
                            r.receiver[0].nr2 = true;
                            button.set_label("NR2");
                            r.receiver[0].set_nr(); // turn off
                            r.receiver[0].set_nr2(); // turn on
                        } else if r.receiver[0].nr2 {
                            r.receiver[0].nr2 = false;
                            button.set_label("NR");
                            button.remove_css_class("active-button");
                            button.add_css_class("inactive-button");
                            r.receiver[0].set_nr2(); // turn off
                        }
                    });

                    let radio_mutex_clone = radio_mutex.clone();
                    app_widgets.nb_button.clone().connect_clicked(move |button| {
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        if !(r.receiver[0].nb | r.receiver[0].nb2) {
                            r.receiver[0].nb = true;
                            button.remove_css_class("inactive-button");
                            button.add_css_class("active-button");
                            r.receiver[0].set_nb(); // turn on
                        } else if r.receiver[0].nb {
                            r.receiver[0].nb = false;
                            r.receiver[0].nb2 = true;
                            button.set_label("NB2");
                            r.receiver[0].set_nb(); // turn off
                            r.receiver[0].set_nb2(); // turn on
                        } else if r.receiver[0].nb2 {
                            r.receiver[0].nb2 = false;
                            button.set_label("NB");
                            button.remove_css_class("active-button");
                            button.add_css_class("inactive-button");
                            r.receiver[0].set_nb2(); // turn off
                        }
                    });

                    let radio_mutex_clone = radio_mutex.clone();
                    app_widgets.anf_button.clone().connect_clicked(move |button| {
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        if r.receiver[0].anf {
                            r.receiver[0].anf = false;
                            button.remove_css_class("active-button");
                            button.add_css_class("inactive-button");
                        } else {
                            r.receiver[0].anf = true;
                            button.remove_css_class("inactive-button");
                            button.add_css_class("active-button");
                        }
                        r.receiver[0].set_anf();
                    });

                    let radio_mutex_clone = radio_mutex.clone();
                    app_widgets.snb_button.clone().connect_clicked(move |button| {
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        if r.receiver[0].snb {
                            r.receiver[0].snb = false;
                            button.remove_css_class("active-button");
                            button.add_css_class("inactive-button");
                        } else {
                            r.receiver[0].snb = true;
                            button.remove_css_class("inactive-button");
                            button.add_css_class("active-button");
                        }
                        r.receiver[0].set_snb();
                    });

                    let radio_mutex_clone = radio_mutex.clone();
                    let rc_app_widgets_clone_clone = rc_app_widgets_clone.clone();
                    app_widgets.mox_button.clone().connect_clicked(move |button| {
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        let app_widgets = rc_app_widgets_clone_clone.borrow_mut();
                        r.mox = button.is_active();
                        if r.mox {
                            button.remove_css_class("inactive-tx-button");
                            button.add_css_class("active-tx-button");
                            if app_widgets.tun_button.is_active() {
                               app_widgets.tun_button.set_active(false);
                               app_widgets.tun_button.remove_css_class("active-tx-button");
                               app_widgets.tun_button.add_css_class("inactive-tx-button");
                               r.tune = false;
                               r.transmitter.set_tuning(r.tune, r.cw_keyer_sidetone_frequency);
                            }
                        } else {
                            button.remove_css_class("active-tx-button");
                            button.add_css_class("inactive-tx-button");
                        }
                        r.updated = true;
                        r.set_state();
                    });

                    let radio_mutex_clone = radio_mutex.clone();
                    let rc_app_widgets_clone_clone = rc_app_widgets_clone.clone();
                    app_widgets.tun_button.clone().connect_clicked(move |button| {
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        let app_widgets = rc_app_widgets_clone_clone.borrow_mut();
                        r.tune = button.is_active();
                        if r.tune {
                            button.remove_css_class("inactive-tx-button");
                            button.add_css_class("active-tx-button");
                            if app_widgets.mox_button.is_active() {
                               app_widgets.mox_button.set_active(false);
                               app_widgets.mox_button.remove_css_class("active-tx-button");
                               app_widgets.mox_button.add_css_class("inactive-tx-button");
                               r.mox = false;
                            }
                        } else {
                            button.remove_css_class("active-tx-button");
                            button.add_css_class("inactive-tx-button");
                        }
                        r.transmitter.set_tuning(r.tune, r.cw_keyer_sidetone_frequency);
                        r.updated = true;
                        r.set_state();
                    });

                    let radio_mutex_clone = radio_mutex.clone();
                    app_widgets.afgain_adjustment.connect_value_changed(move |adjustment| {
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        r.receiver[0].afgain = (adjustment.value() / 100.0) as f32;
                        r.receiver[0].set_afgain();
                    });

                    let radio_mutex_clone = radio_mutex.clone();
                    app_widgets.agc_dropdown.connect_selected_notify(move |dropdown| {
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        let index = dropdown.selected();
                        r.receiver[0].agc = AGC::from_i32(index as i32).expect("Invalid AGC");
                        AGC::set_agc(&r.receiver[0], r.receiver[0].channel);
                    });

                    let radio_mutex_clone = radio_mutex.clone();
                    app_widgets.agcgain_adjustment.connect_value_changed(move |adjustment| {
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        r.receiver[0].agcgain = adjustment.value() as f32;
                        r.receiver[0].set_agcgain();
                    });

                    let radio_mutex_clone = radio_mutex.clone();
                    app_widgets.micgain_adjustment.connect_value_changed(move |adjustment| {
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        r.transmitter.micgain = adjustment.value() as f32;
                        r.transmitter.set_micgain();
                    });

                    let radio_mutex_clone = radio_mutex.clone();
                    app_widgets.drive_adjustment.connect_value_changed(move |adjustment| {
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        r.transmitter.drive = adjustment.value() as f32;
                    });

                    // initialize buttons
                    {
                        let mut r = radio_mutex.radio.lock().unwrap();
                        filter_grid.update_filter_buttons(r.receiver[0].mode);
                        //r.receiver[0].init();
                        r.audio.init();
                        r.receiver[0].set_mode();
                        r.transmitter.init();

                        let mut f = r.receiver[0].frequency_a;
                        if r.receiver[0].ctun {
                            f = r.receiver[0].ctun_frequency;
                        }
                        let formatted_value = format_u32_with_separators(f as u32);
                        app_widgets.vfo_a_frequency.set_label(&formatted_value);

                        if r.receiver[0].nr | r.receiver[0].nr2 {
                            app_widgets.nr_button.add_css_class("active_button");
                        } else {
                            app_widgets.nr_button.add_css_class("inactive_button");
                        }
                        if r.receiver[0].nr2 {
                            app_widgets.nr_button.set_label("NR2");
                        }

                        if r.receiver[0].nb | r.receiver[0].nb2 {
                            app_widgets.nb_button.add_css_class("active_button");
                        } else {
                            app_widgets.nb_button.add_css_class("inactive_button");
                        }
                        if r.receiver[0].nb2 {
                            app_widgets.nb_button.set_label("NR2");
                        }

                        if r.receiver[0].anf {
                            app_widgets.anf_button.add_css_class("active_button");
                        } else {
                            app_widgets.anf_button.add_css_class("inactive_button");
                        }

                        if r.receiver[0].snb {
                            app_widgets.snb_button.add_css_class("active_button");
                        } else {
                            app_widgets.snb_button.add_css_class("inactive_button");
                        }
                        app_widgets.mox_button.add_css_class("inactive_button");
                        app_widgets.tun_button.add_css_class("inactive_button");
                    }   

                    let rc_spectrum_clone2 = rc_spectrum_clone.clone();
                    app_widgets.spectrum_display.set_draw_func(move |_da, cr, width, height| {
                        let mut spectrum = rc_spectrum_clone2.borrow_mut();
                        spectrum.draw(cr, width, height);
                    });

                    let rc_waterfall_clone2 = rc_waterfall_clone.clone();
                    app_widgets.waterfall_display.set_draw_func(move |_da, cr, width, height| {
                        let mut waterfall = rc_waterfall_clone2.borrow_mut();
                        waterfall.draw(cr, width, height);
                    });

                    let rc_meter_clone2 = rc_meter_clone.clone();
                    app_widgets.meter_display.set_draw_func(move |_da, cr, width, height| {
                        let mut meter = rc_meter_clone2.borrow_mut();
                        meter.draw(cr);
                    });

                    match device.protocol {
                        1 => {
                            let mut p1 = Protocol1::new(device);
                            let radio_mutex_clone = radio_mutex.clone();
                            thread::spawn(move || {
                                p1.run(&radio_mutex_clone);
                            });
                        },
                        2 => {
                            let mut p2 = Protocol2::new(device);
                            let radio_mutex_clone = radio_mutex.clone();
                            thread::spawn(move || {
                                p2.run(&radio_mutex_clone);
                            });
                        },
                        _ => eprintln!("Invalid protocol"),
                    }

                    let radio_mutex_clone = radio_mutex.clone();
                    app_widgets.main_window.connect_close_request(move |_| {
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        r.save(device);
                        Propagation::Proceed
                    });

                    let mut update_interval = 100.0;
                    {
                        let r = radio_mutex.radio.lock().unwrap();
                        update_interval = 1000.0 / r.receiver[0].spectrum_fps;
                    }

                    let radio_mutex_clone = radio_mutex.clone();
                    let rc_app_widgets_clone2 = rc_app_widgets_clone.clone();
                    let mut rc_spectrum_clone2 = rc_spectrum_clone.clone();
                    let spectrum_timeout_id = timeout_add_local(Duration::from_millis(update_interval as u64), move || {
                        spectrum_update(&radio_mutex_clone, &rc_app_widgets_clone2, &rc_spectrum_clone2);
                        Continue
                    });

                    {
                        let r = radio_mutex.radio.lock().unwrap();
                        update_interval = 1000.0 / r.receiver[0].waterfall_fps;
                    }
                    let radio_mutex_clone = radio_mutex.clone();
                    let rc_app_widgets_clone2 = rc_app_widgets_clone.clone();
                    let mut rc_waterfall_clone2 = rc_waterfall_clone.clone();
                    let waterfall_timeout_id = timeout_add_local(Duration::from_millis(update_interval as u64), move || {
                        waterfall_update(&radio_mutex_clone, &rc_app_widgets_clone2, &rc_waterfall_clone2);
                        Continue
                    });

                    let radio_mutex_clone = radio_mutex.clone();
                    let rc_app_widgets_clone2 = rc_app_widgets_clone.clone();
                    let mut rc_meter_clone2 = rc_meter_clone.clone();
                    let meter_timeout_id = timeout_add_local(Duration::from_millis(update_interval as u64), move || {
                        meter_update(&radio_mutex_clone, &rc_app_widgets_clone2, &rc_meter_clone2);
                        Continue
                    });


                    {
                        let mut r = radio_mutex.radio.lock().unwrap();
                        r.spectrum_timeout_id = Some(spectrum_timeout_id);
                        r.waterfall_timeout_id = Some(waterfall_timeout_id);
                        r.meter_timeout_id = Some(meter_timeout_id);
                    }

                } else {
                    // try again
                }
            },
            None => {app_clone.quit();},
        }
        Propagation::Proceed
    });

    discovery_dialog.present();
    discovery_dialog.grab_focus();

    let app_widgets = rc_app_widgets.borrow();
    app_widgets.main_window.present();
}

fn spectrum_update(radio_mutex: &RadioMutex,  rc_app_widgets: &Rc<RefCell<AppWidgets>>, rc_spectrum: &Rc<RefCell<Spectrum>>) {
    let app_widgets = rc_app_widgets.borrow();
    let (flag, pixels) = radio_mutex.update_spectrum(app_widgets.waterfall_display.width());
    if flag != 0 {
        let mut spectrum = rc_spectrum.borrow_mut();
        spectrum.update(app_widgets.spectrum_display.width(), app_widgets.spectrum_display.height(), &radio_mutex, &pixels);
        app_widgets.spectrum_display.queue_draw();
    }

}

fn waterfall_update(radio_mutex: &RadioMutex,  rc_app_widgets: &Rc<RefCell<AppWidgets>>, rc_waterfall: &Rc<RefCell<Waterfall>>) {
    let app_widgets = rc_app_widgets.borrow();
    let (flag, pixels) = radio_mutex.update_waterfall(app_widgets.waterfall_display.width());
    if flag != 0 {
        let mut waterfall = rc_waterfall.borrow_mut();
        waterfall.update(app_widgets.waterfall_display.width(), app_widgets.waterfall_display.height(), &radio_mutex, &pixels);
        app_widgets.waterfall_display.queue_draw();
    }
}

fn meter_update(radio_mutex: &RadioMutex,  rc_app_widgets: &Rc<RefCell<AppWidgets>>, rc_meter: &Rc<RefCell<Meter>>) {
    let app_widgets = rc_app_widgets.borrow();
    let mut meter = rc_meter.borrow_mut();
    let mut r = radio_mutex.radio.lock().unwrap();
    if r.is_transmitting() {
    } else {
        unsafe {
            r.s_meter_dbm = GetRXAMeter(r.receiver[0].channel,rxaMeterType_RXA_S_AV as i32);
            if r.receiver[0].subrx {
                r.subrx_s_meter_dbm = GetRXAMeter(r.receiver[0].subrx_channel,rxaMeterType_RXA_S_AV as i32);
            }
        }
        meter.update_rx(r.s_meter_dbm, false);
        if r.receiver[0].subrx {
            meter.update_rx(r.subrx_s_meter_dbm, true);
        }
        app_widgets.meter_display.queue_draw();
    }
}

fn spectrum_waterfall_clicked(radio_mutex: &RadioMutex, rc_app_widgets: &Rc<RefCell<AppWidgets>>, x: f64, width: i32, button: u32) {
    let mut r = radio_mutex.radio.lock().unwrap();
    let app_widgets = rc_app_widgets.borrow();
        
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
        app_widgets.vfo_b_frequency.set_label(&formatted_value);
    } else if r.receiver[0].ctun {
        r.receiver[0].ctun_frequency = f1;
        r.receiver[0].set_ctun_frequency();
        let formatted_value = format_u32_with_separators(r.receiver[0].ctun_frequency as u32);
        app_widgets.vfo_a_frequency.set_label(&formatted_value);
    } else {
        r.receiver[0].frequency_a = f1;
        if r.receiver[0].subrx {
            if r.receiver[0].frequency_b < frequency_low || r.receiver[0].frequency_b > frequency_high {
                r.receiver[0].subrx = false;
                app_widgets.subrx_button.set_active(r.receiver[0].subrx);
            } else {
                r.receiver[0].set_subrx_frequency();
            }
        }
        let formatted_value = format_u32_with_separators(r.receiver[0].frequency_a as u32);
        app_widgets.vfo_a_frequency.set_label(&formatted_value);
    }
}

fn spectrum_waterfall_scroll(radio_mutex: &RadioMutex, rc_app_widgets: &Rc<RefCell<AppWidgets>>, dy: f64) {
    let mut r = radio_mutex.radio.lock().unwrap();
    let app_widgets = rc_app_widgets.borrow();

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
        app_widgets.vfo_a_frequency.set_label(&formatted_value);
        r.receiver[0].set_ctun_frequency();
    } else {
        r.receiver[0].frequency_a = r.receiver[0].frequency_a - (r.receiver[0].step * dy as f32);
        if r.receiver[0].subrx {
            if r.receiver[0].frequency_b < frequency_low || r.receiver[0].frequency_b > frequency_high {
                r.receiver[0].subrx = false;
                app_widgets.subrx_button.set_active(r.receiver[0].subrx);
            } else {
                r.receiver[0].set_subrx_frequency();
            }
        }
        let formatted_value = format_u32_with_separators(r.receiver[0].frequency_a as u32);
        app_widgets.vfo_a_frequency.set_label(&formatted_value);
    }
}

