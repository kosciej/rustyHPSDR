use gtk::prelude::*;
use gtk::{Adjustment, Align, Application, ApplicationWindow, Button, CellRendererText, CheckButton, ComboBox, ComboBoxText,  DrawingArea, Entry, Frame, Grid, Label, ListBox, ListBoxRow, ListStore, Orientation, Scale, ScrolledWindow, SpinButton, ToggleButton, TreeModel, Widget, Window};
use gtk::{EventController, EventControllerScroll, EventControllerScrollFlags, GestureClick};
use gtk::glib::Propagation;
use gtk::cairo::{Context, LineCap, LineJoin, LinearGradient}; 
use gdk_pixbuf::Pixbuf;
use gdk_pixbuf::Colorspace;
use glib::ControlFlow::Continue;

use std::cell::RefCell;
use std::cmp::min;
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
use crate::filters::Filters;
use crate::filters::FilterGrid;
use crate::agc::AGC;
use crate::receiver::Receiver;
use crate::configure::*;
use crate::wdsp::*;
use crate::protocol1::Protocol1;
use crate::protocol2::Protocol2;

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

#[derive(Serialize, Deserialize, Clone)]
pub struct Radio {
    pub name: String,
    pub supported_receivers: u8,
    pub receivers: u8,
    pub receiver: Vec<Receiver>,
    pub band_info: Vec<BandInfo>,
}

impl Radio {

    // currently only supports 1 receiver
    pub fn new(device: Device) -> Radio {
        let name = "?".to_string();
        let supported_receivers = device.supported_receivers;
        let receivers: u8 = 1;
        let mut receiver: Vec<Receiver> = Vec::new();
        for i in 0..receivers {
            receiver.push(Receiver::new(i));
        }
        let band_info = BandInfo::new();
        let radio = Radio {
            name,
            supported_receivers,
            receivers,
            receiver,
            band_info
        };

        radio
    }

    pub fn run(radio: &Arc<Mutex<Radio>>, main_window: &ApplicationWindow, device: Device) {

        //let mut r = radio.lock().unwrap();
        let content = gtk::Box::new(Orientation::Vertical, 0);

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

        // add the main window grid
        content.append(&main_grid);

        // setup default information
        //let mut rx = Arc::new(Mutex::new(r.receiver[0].clone()));
        let band_info = Rc::new(RefCell::new(BandInfo::new()));

        let mut configure_button = Button::with_label("Configure");
        let main_window_for_configure = main_window.clone();
        let band_info_for_configure = band_info.clone();
        let radio_for_configure = Arc::clone(&radio);
        configure_button.connect_clicked(move |_| {
            let main_window_for_dialog = main_window_for_configure.clone();
            let band_info_for_dialog = band_info_for_configure.clone();
            let radio_for_dialog = Arc::clone(&radio_for_configure);
            let configure_dialog = create_configure_dialog(&main_window_for_dialog, band_info_for_dialog, &radio_for_dialog);
            configure_dialog.present();
        });

        main_grid.attach(&configure_button, 0, 0, 1, 1);

        let vfo_a_frame = Frame::new(Some("VFO A"));
        main_grid.attach(&vfo_a_frame, 1, 0, 2, 1);

        let vfo_a_frequency = Label::new(Some("00.000.000"));
        vfo_a_frequency.set_css_classes(&["vfo-a-label"]);
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
            .active-button {
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
             .inactive-button {
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
        main_grid.attach(&vfo_grid, 3, 0, 2, 1);

        let button_a_to_b = Button::with_label("A>B");
        vfo_grid.attach(&button_a_to_b, 0, 0, 1, 1);

        let button_b_to_a = Button::with_label("A<B");
        vfo_grid.attach(&button_b_to_a, 0, 1, 1, 1);

        let button_a_swap_b = Button::with_label("A<>B");
        vfo_grid.attach(&button_a_swap_b, 1, 0, 1, 1);

        let button_split = Button::with_label("SPLIT");
        vfo_grid.attach(&button_split, 1, 1, 1, 1);

        let button_ctun = ToggleButton::with_label("CTUN");
        {
            let r = radio.lock().unwrap();
            button_ctun.set_active(r.receiver[0].ctun);
        }
        let radio_for_ctun = Arc::clone(&radio);
        let vfo_a_frequency_for_ctun = vfo_a_frequency.clone();
        button_ctun.connect_clicked(move |_| {
            let mut r = radio_for_ctun.lock().unwrap();
            if r.receiver[0].ctun {
                r.receiver[0].ctun_frequency = 0.0;
                r.receiver[0].ctun = false;
                unsafe {
                    SetRXAShiftRun(r.receiver[0].channel, 0);
                }
                let formatted_value = format_u32_with_separators(r.receiver[0].frequency_a as u32);
                vfo_a_frequency_for_ctun.set_label(&formatted_value);
            } else {
                r.receiver[0].ctun_frequency = r.receiver[0].frequency_a;
                r.receiver[0].ctun = true;
                unsafe {
                    SetRXAShiftRun(r.receiver[0].channel, 1);
                    SetRXAShiftFreq(r.receiver[0].channel, 0.0);
                    RXANBPSetShiftFrequency(r.receiver[0].channel, 0.0);
                }
            }
        });

        
        vfo_grid.attach(&button_ctun, 2, 0, 1, 1);

        let vfo_b_frame = Frame::new(Some("VFO B"));
        main_grid.attach(&vfo_b_frame, 5, 0, 2, 1);

        let vfo_b_frequency = Label::new(Some("14.150.000"));
        vfo_b_frequency.set_css_classes(&["vfo-b-label"]);
        vfo_b_frame.set_child(Some(&vfo_b_frequency));
        {
            let r = radio.lock().unwrap();
            let formatted_value = format_u32_with_separators(r.receiver[0].frequency_b as u32);
            vfo_b_frequency.set_label(&formatted_value);
        }

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
                let offset = r.receiver[0].ctun_frequency - r.receiver[0].frequency_a;
                unsafe {
                    SetRXAShiftFreq(r.receiver[0].channel, offset.into());
                    RXANBPSetShiftFrequency(r.receiver[0].channel, offset.into());
                }
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
                let offset = r.receiver[0].ctun_frequency - r.receiver[0].frequency_a;
                unsafe {
                    SetRXAShiftFreq(r.receiver[0].channel, offset.into());
                    RXANBPSetShiftFrequency(r.receiver[0].channel, offset.into());
                }
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

        step_combo.set_active(Some(7)); // 1KHz
        tune_step_frame.set_child(Some(&step_combo));

        let meter_frame = Frame::new(Some("S Meter"));
        let meter_label = Label::new(Some("-121 dBm"));
        meter_frame.set_child(Some(&meter_label));

        main_grid.attach(&meter_frame, 9, 0, 2, 1);

        
        let spectrum_display = DrawingArea::new();
        spectrum_display.set_hexpand(true);
        spectrum_display.set_vexpand(true);
        spectrum_display.set_content_width(1024);
        spectrum_display.set_content_height(250);
  

        let scroll_controller_spectrum = EventControllerScroll::new(
            EventControllerScrollFlags::VERTICAL | EventControllerScrollFlags::KINETIC
        );
        let radio_clone = Arc::clone(&radio);
        let f = vfo_a_frequency.clone();
        scroll_controller_spectrum.connect_scroll(move |controller, dx, dy| {
            spectrum_waterfall_scroll(&radio_clone, &f, dy);
            Propagation::Proceed
        });
        spectrum_display.add_controller(scroll_controller_spectrum.clone());

        let scroll_controller_a = EventControllerScroll::new(
            EventControllerScrollFlags::VERTICAL
        );
        let radio_clone = Arc::clone(&radio);
        let f = vfo_a_frequency.clone();
        scroll_controller_a.connect_scroll(move |controller, dx, dy| {
            spectrum_waterfall_scroll(&radio_clone, &f, dy);
            Propagation::Proceed
        });
        vfo_a_frequency.add_controller(scroll_controller_a);

        //let click_gesture = GestureClick::new();
        let spectrum_click_gesture = Rc::new(GestureClick::new());
        let spectrum_click_gesture_clone_for_callback = spectrum_click_gesture.clone();
        let radio_clone = Arc::clone(&radio);
        let f = vfo_a_frequency.clone();
        spectrum_click_gesture_clone_for_callback.connect_pressed(move |gesture, _, x, y| {
            let da = gesture.widget().unwrap();
            let width = da.allocated_width();
            spectrum_waterfall_clicked(&radio_clone, &f, x, width);
        });
        spectrum_display.add_controller(<GestureClick as Clone>::clone(&spectrum_click_gesture).upcast::<EventController>());

        let radio_clone = Arc::clone(&radio);
        spectrum_display.connect_resize(move |_, width, height| {
            println!("Spectrum resized to: {}x{}", width, height);
/*
            let mut r = rx_clone.lock().unwrap();
            println!("Spectrum resized to: {}x{}", width, height);
            r.spectrum_width = width;
            r.init_analyzer();
*/
        });
        main_grid.attach(&spectrum_display, 1, 1, 10, 4);


        let pixbuf: Rc<RefCell<Option<Pixbuf>>> = Rc::new(RefCell::new(None));
        let waterfall_display = DrawingArea::new();
        waterfall_display.set_hexpand(true);
        waterfall_display.set_vexpand(true);
        waterfall_display.set_content_width(1024);
        waterfall_display.set_content_height(250);

        let scroll_controller_waterfall = EventControllerScroll::new(
            EventControllerScrollFlags::VERTICAL
        );
        let radio_clone = Arc::clone(&radio);
        let f = vfo_a_frequency.clone();
        scroll_controller_waterfall.connect_scroll(move |controller, dx, dy| {
            spectrum_waterfall_scroll(&radio_clone, &f, dy);
            Propagation::Proceed
        });
        waterfall_display.add_controller(scroll_controller_waterfall.clone());

        let waterfall_click_gesture = Rc::new(GestureClick::new());
        let waterfall_click_gesture_clone_for_callback = waterfall_click_gesture.clone();
        let radio_clone = Arc::clone(&radio);
        let f = vfo_a_frequency.clone();
        waterfall_click_gesture_clone_for_callback.connect_pressed(move |gesture, _, x, y| {
            let da = gesture.widget().unwrap();
            let width = da.allocated_width();
            spectrum_waterfall_clicked(&radio_clone, &f, x, width);
        });
        waterfall_display.add_controller(<GestureClick as Clone>::clone(&waterfall_click_gesture).upcast::<EventController>());

        let radio_clone = Arc::clone(&radio);
        waterfall_display.connect_resize(move |_, width, height| {
            println!("Waterfall resized to: {}x{}", width, height);
            let mut r = radio_clone.lock().unwrap();
            println!("Waterfall resized to: {}x{}", width, height);
            r.receiver[0].spectrum_width = width;
            r.receiver[0].init_analyzer();
        });
        main_grid.attach(&waterfall_display, 1, 5, 10, 4);

        let band_frame = Frame::new(Some("Band"));
        main_grid.attach(&band_frame, 11, 0, 2, 2);
        let band_info_clone = band_info.clone();
        let band_grid = Rc::new(RefCell::new(BandGrid::new(band_info_clone)));
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
        let band_info_for_callback = band_info.clone();
        let mode_grid_for_callback = mode_grid.clone();
        let filter_grid_for_callback = filter_grid.clone();
        let radio_for_callback = Arc::clone(&radio);
        let vfo_a_frequency_for_callback = vfo_a_frequency.clone();
        let r = radio.lock().unwrap();
        let band = r.receiver[0].band.to_i32() as usize;
        drop(r);
        band_grid_for_callback.set_callback(move|index| {
            let mut r = radio_for_callback.lock().unwrap();
            r.receiver[0].band = Bands::from_u32(index as u32).expect("Failed to get band from index");
            let band_info = &band_info_for_callback.borrow()[index];
            r.receiver[0].frequency_a = band_info.low + ((band_info.high - band_info.low) / 2.0);
            r.receiver[0].spectrum_low = band_info.spectrum_low;
            r.receiver[0].spectrum_high = band_info.spectrum_high;
            r.receiver[0].waterfall_low = band_info.waterfall_low;
            r.receiver[0].waterfall_high = band_info.waterfall_high;
            if r.receiver[0].ctun {
                r.receiver[0].ctun_frequency = r.receiver[0].frequency_a;
                unsafe {
                    SetRXAShiftFreq(r.receiver[0].channel, 0.0);
                    RXANBPSetShiftFrequency(r.receiver[0].channel, 0.0);
                }
            }

            if !r.receiver[0].filters_manual {
                r.receiver[0].filters = band_info_for_callback.borrow()[index].filters;
            }

            let filter_grid_mut = filter_grid_for_callback.borrow_mut();
            let mode_grid_mut = mode_grid_for_callback.borrow_mut();
            mode_grid_mut.set_active_index(band_info.mode.to_usize());
            filter_grid_mut.update_filter_buttons(band_info.mode.to_usize());
            filter_grid_mut.set_active_index(band_info.filter.to_usize());
            r.receiver[0].mode = band_info.mode.to_usize();
            let (low, high) = filter_grid_mut.get_filter_values(band_info.mode.to_usize(), band_info.filter.to_usize());

            r.receiver[0].filter_low = low;
            r.receiver[0].filter_high = high;
            unsafe {
                SetRXAMode(r.receiver[0].channel, r.receiver[0].mode as i32);
                RXASetPassband(r.receiver[0].channel,r.receiver[0].filter_low.into(),r.receiver[0].filter_high.into());
            }

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
            let mut radio = radio_for_mode_set_callback.lock().unwrap();
            radio.receiver[0].mode = index;
            let filter_grid = filter_grid_for_mode_set_callback.borrow_mut();
            filter_grid.update_filter_buttons(index);

            let (low, high) = filter_grid.get_filter_values(index, 5);
            radio.receiver[0].filter_low = low;
            radio.receiver[0].filter_high = high;
            unsafe {
                SetRXAMode(radio.receiver[0].channel, radio.receiver[0].mode as i32);
                RXASetPassband(radio.receiver[0].channel,radio.receiver[0].filter_low.into(),radio.receiver[0].filter_high.into());
            }

        }, mode);

        let r = radio.lock().unwrap();
        let filter = r.receiver[0].filter;
        drop(r);
        let radio_for_filter_callback = Arc::clone(&radio);
        let filter_grid_for_set_callback = filter_grid.clone();
        let mut filter_grid_for_callback = filter_grid.borrow_mut();
        filter_grid_for_callback.set_callback(move|index| {
            let mut radio = radio_for_filter_callback.lock().unwrap();
            let filter_grid = filter_grid_for_set_callback.borrow();
            radio.receiver[0].filter = index;
            let (low, high) = filter_grid.get_filter_values(radio.receiver[0].mode, radio.receiver[0].filter);
            radio.receiver[0].filter_low = low;
            radio.receiver[0].filter_high = high;
            unsafe {
                SetRXAMode(radio.receiver[0].channel, radio.receiver[0].mode as i32);
                RXASetPassband(radio.receiver[0].channel,radio.receiver[0].filter_low.into(),radio.receiver[0].filter_high.into());
            }
        }, filter);


        let mut r = radio.lock().unwrap();

        let afgain_frame = Frame::new(Some("AF Gain"));
        main_grid.attach(&afgain_frame, 0, 1, 1, 1);
        let afgain_adjustment = Adjustment::new(
            (r.receiver[0].afgain * 100.0).into(), // Initial value
            0.0,  // Minimum value
            100.0, // Maximum value
            1.0,  // Step increment
            1.0, // Page increment
            0.0,  // Page size (not typically used for simple scales)
        );
        let afgain_scale = Scale::new(Orientation::Horizontal, Some(&afgain_adjustment));
        afgain_scale.set_digits(0); // Display whole numbers
        afgain_scale.set_draw_value(true); // Display the current value next to the slider
        afgain_frame.set_child(Some(&afgain_scale));

        let afgain_radio = Arc::clone(&radio);
        afgain_adjustment.connect_value_changed(move |adjustment| {
            let mut r = afgain_radio.lock().unwrap();
            r.receiver[0].afgain = (adjustment.value() / 100.0) as f32;
            unsafe {
                SetRXAPanelGain1(r.receiver[0].channel, r.receiver[0].afgain.into());
            }
        });

        let agcgain_frame = Frame::new(Some("AGC Gain"));
        main_grid.attach(&agcgain_frame, 0, 2, 1, 1);
        let agcgain_adjustment = Adjustment::new(
            r.receiver[0].agcgain.into(), // Initial value
            -20.0,  // Minimum value
            120.0, // Maximum value
            1.0,  // Step increment
            1.0, // Page increment
            0.0,  // Page size (not typically used for simple scales)
        );
        let agcgain_scale = Scale::new(Orientation::Horizontal, Some(&agcgain_adjustment));
        agcgain_scale.set_digits(0); // Display whole numbers
        agcgain_scale.set_draw_value(true); // Display the current value next to the slider
        agcgain_frame.set_child(Some(&agcgain_scale));

        let agcgain_radio = Arc::clone(&radio);
        agcgain_adjustment.connect_value_changed(move |adjustment| {
            let mut r = agcgain_radio.lock().unwrap();
            r.receiver[0].agcgain = adjustment.value() as f32;
            unsafe {
                SetRXAAGCTop(r.receiver[0].channel, r.receiver[0].agcgain.into());
            }
        });


        let micgain_frame = Frame::new(Some("Mic Gain"));
        main_grid.attach(&micgain_frame, 0, 3, 1, 1);
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
        main_grid.attach(&drive_frame, 0, 4, 1, 1);
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


        let agc_frame = Frame::new(Some("AGC"));
        main_grid.attach(&agc_frame, 0, 5, 1, 1);

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

        agc_combo.set_active(Some(r.receiver[0].agc as u32));
        agc_frame.set_child(Some(&agc_combo));

        let agc_combo_radio = Arc::clone(&radio);
        agc_combo.connect_changed(move |agc_combo| {
            let mut r = agc_combo_radio.lock().unwrap();
            let index = agc_combo.active().unwrap_or(0);
            r.receiver[0].agc = AGC::from_i32(index as i32).expect("Invalid AGC");
            AGC::set_agc(&r.receiver[0]);
        });

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
        main_grid.attach(&noise_grid, 1, 9, 2, 1);

        let button_nr = ToggleButton::with_label("NR2");
        button_nr.set_active(r.receiver[0].nr);
        noise_grid.attach(&button_nr, 0, 0, 1, 1);
        let radio_for_nr = Arc::clone(&radio);
        button_nr.connect_clicked(move |_| {
            let mut r = radio_for_nr.lock().unwrap();
            if r.receiver[0].nr {
                r.receiver[0].nr = false;
            } else {
                r.receiver[0].nr = true;
            }
            unsafe {
                SetRXAEMNRRun(r.receiver[0].channel, r.receiver[0].nr as i32);
            }
        });

        let button_nb = ToggleButton::with_label("NB2");
        button_nb.set_active(r.receiver[0].nb);
        noise_grid.attach(&button_nb, 0, 1, 1, 1);
        let radio_for_nb = Arc::clone(&radio);
        button_nb.connect_clicked(move |_| {
            let mut r = radio_for_nb.lock().unwrap();
            if r.receiver[0].nb {
                r.receiver[0].nb = false;
            } else {
                r.receiver[0].nb = true;
            }
            unsafe {
                SetEXTNOBRun(r.receiver[0].channel, r.receiver[0].nb as i32);
            }
        });

        let button_anf = ToggleButton::with_label("ANF");
        button_anf.set_active(r.receiver[0].anf);
        noise_grid.attach(&button_anf, 1, 0, 1, 1);
        let radio_for_anf = Arc::clone(&radio);
        button_anf.connect_clicked(move |_| {
            let mut r = radio_for_anf.lock().unwrap();
            if r.receiver[0].anf {
                r.receiver[0].anf = false;
            } else { 
                r.receiver[0].anf = true;
            }
            unsafe {
                SetRXAANFRun(r.receiver[0].channel, r.receiver[0].anf as i32);
            }
        });

        let button_snb = ToggleButton::with_label("SNB");
        button_snb.set_active(r.receiver[0].snb);
        noise_grid.attach(&button_snb, 1, 1, 1, 1);
        let radio_for_snb = Arc::clone(&radio);
        button_snb.connect_clicked(move |_| {
            let mut r = radio_for_snb.lock().unwrap();
            if r.receiver[0].snb {
                r.receiver[0].snb = false;
            } else {
                r.receiver[0].snb = true;
            }
            unsafe {
                SetRXASNBARun(r.receiver[0].channel, r.receiver[0].snb as i32);
            } 
        });


        r.receiver[0].init();

        drop(r);

        main_window.set_child(Some(&content));

        let radio_clone_for_protocol = Arc::clone(&radio);
        match device.protocol {
            1 => {
                let mut p1 = Protocol1::new(device, radio_clone_for_protocol);
                let radio_clone_for_spawn = Arc::clone(&radio);
                thread::spawn(move || {
                    p1.run(device, radio_clone_for_spawn);
                });
            },
            2 => {
                let mut p2 = Protocol2::new(device, radio_clone_for_protocol);
                let radio_clone_for_spawn = Arc::clone(&radio);
                thread::spawn(move || {
                    p2.run(device, radio_clone_for_spawn);
                });
            },
            _ => eprintln!("Invalid protocol"),
        }


        let r = radio.lock().unwrap();
        let update_interval = 1000.0 / r.receiver[0].fps;
        drop(r);
        let spectrum_display_for_timeout = spectrum_display.clone();
        let waterfall_display_for_timeout = waterfall_display.clone();
        let radio_clone_for_timeout = Arc::clone(&radio);
        let band_info_for_timeout = band_info.clone();
        let pixbuf_for_timeout = pixbuf.clone();
        glib::timeout_add_local(Duration::from_millis(update_interval as u64), move || {
            let mut pixels = vec![0.0; spectrum_display_for_timeout.width() as usize];
            let mut flag: c_int = 0;
            unsafe {
                let r = radio_clone_for_timeout.lock().unwrap();
                GetPixels(r.receiver[0].channel, 0, pixels.as_mut_ptr(), &mut flag);
            }
            if flag != 0 {
                let radio_clone_for_draw = radio_clone_for_timeout.clone(); 
                let band_info_for_draw = band_info_for_timeout.clone();
                let meter_label_for_draw = meter_label.clone();
                let pixbuf_for_draw = pixbuf_for_timeout.clone();
                let waterfall_display_for_draw = waterfall_display_for_timeout.clone();
                spectrum_display_for_timeout.set_draw_func(move |da, cr, width, height|{
                    {
                        let r = radio_clone_for_draw.lock().unwrap();
                        let band_info = &band_info_for_draw.borrow()[r.receiver[0].band.to_i32() as usize];
                        drop(r);
                        draw_spectrum(da, cr, width, height, &radio_clone_for_draw, &pixels, band_info);
                        let pixbuf_for_waterfall = pixbuf_for_draw.clone();
                        update_waterfall(width, height, &radio_clone_for_draw, &pixbuf_for_waterfall, &pixels);
                    }

                    unsafe {
                        let r = radio_clone_for_draw.lock().unwrap();
                        let meter_db = GetRXAMeter(r.receiver[0].channel,rxaMeterType_RXA_S_AV as i32);
                        let label_text = format!("{} dBm", meter_db as i32);
                        meter_label_for_draw.set_label(&label_text);
                    }

                    let pixbuf_for_waterfall_draw = pixbuf_for_draw.clone();
                    waterfall_display_for_draw.set_draw_func(move |_da, cr, width, height| {
                        draw_waterfall(cr, width, height, &pixbuf_for_waterfall_draw);
                    });
                });

                spectrum_display_for_timeout.queue_draw();
                waterfall_display_for_timeout.queue_draw();
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
                    Err(e) => {
                        Self::new(device)
                    }
                },
                Err(e) => {
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
                if let Err(e) = fs::create_dir_all(parent) {
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

fn spectrum_waterfall_clicked(radio: &Arc<Mutex<Radio>>, f: &Label, x: f64, width: i32) {
    let mut r = radio.lock().unwrap();
        
    if x<20.0 {
        r.receiver[0].spectrum_low = r.receiver[0].spectrum_low - 1.0
    } else if x > (width-20) as f64 {
        r.receiver[0].spectrum_low = r.receiver[0].spectrum_low + 1.0
    } else {
    let frequency_low = r.receiver[0].frequency_a - (r.receiver[0].sample_rate/2) as f32;
    let frequency_high = r.receiver[0].frequency_a + (r.receiver[0].sample_rate/2) as f32;
    let hz_per_pixel=(frequency_high - frequency_low) as f32 / width as f32;
    let f1 = frequency_low + (x as f32 * hz_per_pixel);
    let f1 = (f1 as u32 / r.receiver[0].step as u32 * r.receiver[0].step as u32) as f32;
 
    if r.receiver[0].ctun {
        r.receiver[0].ctun_frequency = f1;
        let offset = r.receiver[0].ctun_frequency - r.receiver[0].frequency_a;
        unsafe {
            SetRXAShiftFreq(r.receiver[0].channel, offset.into());
            RXANBPSetShiftFrequency(r.receiver[0].channel, offset.into());
        }
        let formatted_value = format_u32_with_separators(r.receiver[0].ctun_frequency as u32);
        f.set_label(&formatted_value);
    } else {
        r.receiver[0].frequency_a = f1;
        let formatted_value = format_u32_with_separators(r.receiver[0].frequency_a as u32);
        f.set_label(&formatted_value);
    }
    }
}

fn spectrum_waterfall_scroll(radio: &Arc<Mutex<Radio>>, f: &Label, dy: f64) {
    let mut r = radio.lock().unwrap();
    if r.receiver[0].ctun {
        let frequency_low = r.receiver[0].frequency_a - (r.receiver[0].sample_rate/2) as f32;
        let frequency_high = r.receiver[0].frequency_a + (r.receiver[0].sample_rate/2) as f32;
        r.receiver[0].ctun_frequency = r.receiver[0].ctun_frequency - (r.receiver[0].step * dy as f32);
        if r.receiver[0].ctun_frequency < frequency_low {
            r.receiver[0].ctun_frequency = frequency_low;
        } else if r.receiver[0].ctun_frequency > frequency_high {
            r.receiver[0].ctun_frequency = frequency_high;
        }
        let formatted_value = format_u32_with_separators(r.receiver[0].ctun_frequency as u32);
        f.set_label(&formatted_value);

        let offset = r.receiver[0].ctun_frequency - r.receiver[0].frequency_a;
        unsafe {
            SetRXAShiftFreq(r.receiver[0].channel, offset.into());
            RXANBPSetShiftFrequency(r.receiver[0].channel, offset.into());
        }
    } else {
        r.receiver[0].frequency_a = r.receiver[0].frequency_a - (r.receiver[0].step * dy as f32);
        let formatted_value = format_u32_with_separators(r.receiver[0].frequency_a as u32);
        f.set_label(&formatted_value);
    }
}

fn draw_spectrum(area: &DrawingArea, cr: &Context, width: i32, height: i32, radio: &Arc<Mutex<Radio>>, pixels: &Vec<f32>, band_info: &BandInfo) {
    cr.set_source_rgb(0.0, 0.0, 0.0);
    cr.paint().unwrap();
                      
    let r = radio.lock().unwrap();   
    let dbm_per_line: f32 = height as f32/(r.receiver[0].spectrum_high-r.receiver[0].spectrum_low);

    cr.set_source_rgb(1.0, 1.0, 0.0);                   
    cr.set_line_width(1.0);
    cr.set_line_cap(LineCap::Round);
    cr.set_line_join(LineJoin::Round);

    let frequency_low = r.receiver[0].frequency_a - (r.receiver[0].sample_rate/2) as f32;
    let frequency_high = r.receiver[0].frequency_a + (r.receiver[0].sample_rate/2) as f32;
    let hz_per_pixel=(frequency_high - frequency_low) as f32 / width as f32;
                                
    cr.set_source_rgb(0.5, 0.5, 0.5);
    let mut step = 25000.0;  
    match r.receiver[0].sample_rate {       
         48000 => step = 50000.0,
         96000 => step = 10000.0,   
        192000 => step = 20000.0,
        384000 => step = 25000.0,
        768000 => step = 50000.0,
       1536000 => step = 100000.0,
             _ => step = 25000.0,
    }
   
    // draw the frequency markers
    cr.set_source_rgb(1.0, 1.0, 1.0);
    let mut f: f32 = (((frequency_low as i32 + step as i32) / step as i32) * step as i32) as f32;
    while f < frequency_high {
        let x = (f - frequency_low) / hz_per_pixel;
        cr.move_to( x.into(), 0.0);
        cr.line_to( x.into(), height.into());
        let text = format!("{}", f);
        cr.move_to( x.into(), 20.0);
        let _ = cr.show_text(&text);
        f = f + step as f32;
    }
    cr.stroke().unwrap();

    // draw the band limits
    cr.set_source_rgb(1.0, 0.0, 0.0);
    if frequency_low < band_info.low && frequency_high > band_info.low {
        let x = (band_info.low - frequency_low) / hz_per_pixel;
        cr.move_to( x.into(), 0.0);
        cr.line_to( x.into(), height.into());
    }

    if frequency_low < band_info.high && frequency_high > band_info.high {
        let x = (band_info.high - frequency_low) / hz_per_pixel;
        cr.move_to( x.into(), 0.0);
        cr.line_to( x.into(), height.into());
    }
    cr.stroke().unwrap();


    // draw signal levels
    cr.set_source_rgb(0.5, 0.5, 0.5);
    for i in (r.receiver[0].spectrum_low as i32 .. r.receiver[0].spectrum_high as i32).step_by(r.receiver[0].spectrum_step as usize) {
        let y = (r.receiver[0].spectrum_high - i as f32) * dbm_per_line;
        cr.move_to(0.0, y.into());
        cr.line_to(width as f64, y.into());
        let text = format!("{}dbm", i);
        cr.move_to( 5.0, (y-2.0).into());
        let _ = cr.show_text(&text);
    }
    cr.stroke().unwrap();

    cr.move_to(0.0, height as f64);

    cr.set_source_rgb(1.0, 1.0, 0.0);
    for (i, &pixel) in pixels.iter().enumerate() {
        let mut y = ((r.receiver[0].spectrum_high - pixel as f32) * dbm_per_line).floor();
        cr.line_to(i as f64, y.into());
    }
    cr.line_to(width as f64, height as f64);

    let pattern = LinearGradient::new(0.0, (height-20) as f64, 0.0, 0.0);
    let mut s9: f32 = -73.0;
    s9 = ((r.receiver[0].spectrum_high - s9)
                  * (height-20) as f32
                / (r.receiver[0].spectrum_high - r.receiver[0].spectrum_low)).floor();
    s9 = 1.0-(s9/(height-20) as f32);

    pattern.add_color_stop_rgb(0.0,0.0,1.0,0.0); // Green
    pattern.add_color_stop_rgb((s9/3.0).into(),1.0,0.65,0.0); // Orange
    pattern.add_color_stop_rgb(((s9/3.0)*2.0).into(),1.0,1.0,0.0); // Yellow
    pattern.add_color_stop_rgb(s9.into(),1.0,0.0,0.0); // Red

    cr.set_source(&pattern).expect("Failed to set source");
    cr.close_path();
    let _ = cr.fill_preserve();
    cr.stroke().unwrap();

    let mut offset: f32 = 0.0;
    if r.receiver[0].ctun {
        offset = (r.receiver[0].ctun_frequency - r.receiver[0].frequency_a) / hz_per_pixel;
    }

    // filter
    cr.set_source_rgba (0.5, 0.5, 0.5, 0.50);
    let filter_left=offset+(width as f32/2.0)+(r.receiver[0].filter_low/hz_per_pixel);
    let filter_right=offset+(width as f32/2.0)+(r.receiver[0].filter_high/hz_per_pixel);
    cr.rectangle(filter_left.into(), 0.0, (filter_right-filter_left).into(), height.into());
    let _ = cr.fill();

    // draw the center line frequency marker
    cr.set_source_rgb(1.0, 0.0, 0.0);
    cr.set_line_width(1.0);
    cr.move_to((offset + (width/2) as f32).into(), 0.0);
    cr.line_to((offset + (width/2) as f32).into(), height.into());
    cr.stroke().unwrap();
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

                for y in (0..height - 1).rev() { // Iterate in reverse order
                    let src_offset = (y * row_size) as usize;
                    let dest_offset = ((y + 1) * row_size) as usize;
                    pixels.copy_within(src_offset..src_offset + row_size as usize, dest_offset);
                }

                let w = min(width,new_pixels.len().try_into().unwrap());
                for x in 0..w {
                    let mut value: f32 = (new_pixels[x as usize] as f32);
                    if value < r.receiver[0].waterfall_low {
                        value = r.receiver[0].waterfall_low;
                    } else if value > r.receiver[0].waterfall_high {
                        value = r.receiver[0].waterfall_high;
                    }
                    let percent = 100.0 / ((r.receiver[0].waterfall_high - r.receiver[0].waterfall_low) / (value-r.receiver[0].waterfall_low));
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

