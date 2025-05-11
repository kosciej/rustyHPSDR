use gtk::prelude::*;
use gtk::{Align, ApplicationWindow, Box, Button, Grid, Label, Notebook, Orientation, SpinButton, Window};

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use crate::bands::BandInfo;
use crate::receiver::Receiver;
use crate::radio::Radio;

pub fn create_configure_dialog(parent: &ApplicationWindow, band_info: Rc<RefCell<Vec<BandInfo>>>, radio: &Arc<Mutex<Radio>>) -> Window {

    let window = Window::builder()
        .title("rustyHPSDR Discovery")
        .modal(false)
        .transient_for(parent)
        .destroy_with_parent(true)
        .default_width(800)
        .default_height(200)
        .build();

    let notebook = Notebook::new();

    let display_label = Label::new(Some("Display"));
    let display_grid = Grid::builder()
            .margin_start(0)
            .margin_end(0)
            .margin_top(0)
            .margin_bottom(0)
            .halign(Align::Center)
            .valign(Align::Center)
            .row_spacing(0)
            .column_spacing(0)
            .build();

    display_grid.set_column_homogeneous(true);
    display_grid.set_row_homogeneous(true);

    let band_title = Label::new(Some("Band"));
    display_grid.attach(&band_title, 0, 0, 1, 1);
    let spectrum_low_title = Label::new(Some("Spectrum Low"));
    display_grid.attach(&spectrum_low_title, 1, 0, 1, 1);
    let spectrum_high_title = Label::new(Some("Spectrum High"));
    display_grid.attach(&spectrum_high_title, 2, 0, 1, 1);
    let waterfall_low_title = Label::new(Some("Waterfall Low"));
    display_grid.attach(&waterfall_low_title, 3, 0, 1, 1);
    let waterfall_high_title = Label::new(Some("Waterfall High"));
    display_grid.attach(&waterfall_high_title, 4, 0, 1, 1);

    for (i, info) in band_info.borrow().iter().enumerate() {

        let row = (i+1) as i32;
        let band_label = Label::new(Some(info.label.as_str()));
        display_grid.attach(&band_label, 0, row, 1, 1);

        let spectrum_low_spin_button = SpinButton::with_range(-140.0, -40.0, 1.0);
        spectrum_low_spin_button.set_value(info.spectrum_low.into());
        let info_clone_spectrum_low = band_info.clone();
        let radio_for_spectrum_low = radio.clone();
        spectrum_low_spin_button.connect_value_changed(move |spin_button| {
            info_clone_spectrum_low.borrow_mut()[i].spectrum_low = spin_button.value() as f32;
            let mut r = radio_for_spectrum_low.lock().unwrap();
            if r.receiver[0].band == info_clone_spectrum_low.borrow_mut()[i].band {
                r.receiver[0].spectrum_low = info_clone_spectrum_low.borrow_mut()[i].spectrum_low
            }
        });
        display_grid.attach(&spectrum_low_spin_button, 1, row, 1, 1);

        let spectrum_high_spin_button = SpinButton::with_range(-140.0, -40.0, 1.0);
        spectrum_high_spin_button.set_value(info.spectrum_high.into());
        let info_clone_spectrum_high = band_info.clone();
        let radio_for_spectrum_high = radio.clone();
        spectrum_high_spin_button.connect_value_changed(move |spin_button| {
            info_clone_spectrum_high.borrow_mut()[i].spectrum_high = spin_button.value() as f32;
            let mut r = radio_for_spectrum_high.lock().unwrap();
            if r.receiver[0].band == info_clone_spectrum_high.borrow_mut()[i].band {
                r.receiver[0].spectrum_high = info_clone_spectrum_high.borrow_mut()[i].spectrum_high
            }
        });
        display_grid.attach(&spectrum_high_spin_button, 2, row, 1, 1);

        let waterfall_low_spin_button = SpinButton::with_range(-140.0, -40.0, 1.0);
        waterfall_low_spin_button.set_value(info.waterfall_low.into());
        let info_clone_waterfall_low = band_info.clone();
        let radio_for_waterfall_low = radio.clone();
        waterfall_low_spin_button.connect_value_changed(move |spin_button| {
            info_clone_waterfall_low.borrow_mut()[i].waterfall_low = spin_button.value() as f32;
            let mut r = radio_for_waterfall_low.lock().unwrap();
            if r.receiver[0].band == info_clone_waterfall_low.borrow_mut()[i].band {
                r.receiver[0].waterfall_low = info_clone_waterfall_low.borrow_mut()[i].waterfall_low
            }
        });
        display_grid.attach(&waterfall_low_spin_button, 3, row, 1, 1);

        let waterfall_high_spin_button = SpinButton::with_range(-140.0, -40.0, 1.0);
        waterfall_high_spin_button.set_value(info.waterfall_high.into());
        let info_clone_waterfall_high = band_info.clone();
        let radio_for_waterfall_high = radio.clone();
        waterfall_high_spin_button.connect_value_changed(move |spin_button| {
            info_clone_waterfall_high.borrow_mut()[i].waterfall_high = spin_button.value() as f32;
            let mut r = radio_for_waterfall_high.lock().unwrap();
            if r.receiver[0].band == info_clone_waterfall_high.borrow_mut()[i].band {
                r.receiver[0].waterfall_high = info_clone_waterfall_high.borrow_mut()[i].waterfall_high
            }
        });
        display_grid.attach(&waterfall_high_spin_button, 4, row, 1, 1);

    }

    notebook.append_page(&display_grid, Some(&display_label));

    let button_box = gtk::Box::new(Orientation::Horizontal, 5);
    button_box.set_halign(gtk::Align::End);

    let ok_button = Button::builder().label("Ok").build();

    button_box.append(&ok_button);

    let main_vbox = gtk::Box::new(Orientation::Vertical, 0);
    main_vbox.append(&notebook);
    main_vbox.append(&button_box);
    window.set_child(Some(&main_vbox));

    let window_for_ok = window.clone();
    ok_button.connect_clicked(move |_| {
        window_for_ok.close();
    });


    window
}

