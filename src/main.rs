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
use gtk::{Application, ApplicationWindow};
use gtk::glib::Propagation;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use rustyHPSDR::discovery::Device;
use rustyHPSDR::discovery::discover;
use rustyHPSDR::discovery::create_discovery_dialog;
use rustyHPSDR::radio::Radio;
use rustyHPSDR::receiver::Receiver;

fn main() {
    let application = Application::builder()
        .application_id("org.g0orx.rustyHPSDR")
        .build();

    application.connect_activate(|app| {

        let main_window = ApplicationWindow::builder()
            .application(app)
            .title("rustyHPSDR")
            .build();

            let mut discovery_vec: Vec<Device> = Vec::new();
            discover(&mut discovery_vec);

            let selected_index: Rc<RefCell<Option<i32>>> = Rc::new(RefCell::new(None));

            let main_window_clone = main_window.clone();
            let selected_index_for_discovery_dialog = selected_index.clone();
            let discovery_dialog = create_discovery_dialog(Some(&main_window_clone), &discovery_vec, selected_index_for_discovery_dialog);

            let selected_index_for_close = selected_index.clone();
            let discovery_vec_for_close = discovery_vec.clone();
            let app_for_close = app.clone();
            let main_window_for_close = main_window.clone();
            discovery_dialog.connect_close_request(move |_| {
                let index = *selected_index_for_close.borrow();
                match index {
                    Some(i) => {
                        if i >= 0 {
                            let device = discovery_vec_for_close[(i-1) as usize];
                            let radio = Arc::new(Mutex::new(Radio::load(device)));

                            let mut radio_clone_for_show = radio.clone();
                            let main_window_clone_for_show = main_window_for_close.clone();
                            main_window_for_close.connect_show(move |_| {
                                Radio::run(&radio_clone_for_show, &main_window_clone_for_show, device);
                            });
                    
                            let mut radio_clone_for_close = radio.clone();
                            let main_window_clone_for_close = main_window_for_close.clone();
                            main_window_for_close.connect_close_request(move |_| {
                                let r = radio_clone_for_close.lock().unwrap();
                                r.save(device);
                                Propagation::Proceed
                            });
    
                            main_window_for_close.present();
                        } else {
                            // try again
                        }
                    },
                    None => {
                        println!("None selected");
                        app_for_close.quit();
                    },
                }
                Propagation::Proceed
            });

            discovery_dialog.present();

    });

    application.run();
}
