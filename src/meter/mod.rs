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

use gtk::cairo::{Context, Format, ImageSurface};

use crate::radio::RadioMutex;

#[derive(Clone)]
pub struct Meter {
    surface: ImageSurface,
}

impl Meter {
    pub fn new(width: i32, height: i32) -> Self {
        let surface =
            ImageSurface::create(Format::ARgb32, width, height).expect("Failed to create surface");
        Self { surface }
    }

    pub fn update_rx(&mut self, dbm: f64, rx2: bool) {
        let cr =
            Context::new(self.surface.clone()).expect("Couldn't create cairo context from surface");

        if !rx2 {
            cr.set_source_rgb(0.0, 0.0, 0.0);
            cr.paint().unwrap();
        }
        let x_offset = 5.0;
        let mut y_offset = 10.0;
        if rx2 {
            y_offset = 40.0;
        }
        let db = 2.0; // size in pixels of each dbm

        cr.set_source_rgb(0.0, 1.0, 0.0);
        cr.rectangle(x_offset, 0.0 + y_offset, (dbm + 121.0) * db, 10.0);
        let _ = cr.fill();

        cr.set_source_rgb(1.0, 1.0, 1.0);
        for i in 0..54 {
            cr.move_to(x_offset + (i as f64 * db), 10.0 + y_offset);
            if i % 18 == 0 {
                cr.line_to(x_offset + (i as f64 * db), 0.0 + y_offset);
            } else if i % 6 == 0 {
                cr.line_to(x_offset + (i as f64 * db), 5.0 + y_offset);
            }
        }
        cr.move_to(x_offset + (54.0 * db), 10.0 + y_offset);
        cr.line_to(x_offset + (54.0 * db), 0.0 + y_offset);
        cr.move_to(x_offset + (74.0 * db), 10.0 + y_offset);
        cr.line_to(x_offset + (74.0 * db), 0.0 + y_offset);
        cr.move_to(x_offset + (94.0 * db), 10.0 + y_offset);
        cr.line_to(x_offset + (94.0 * db), 0.0 + y_offset);
        cr.move_to(x_offset + (114.0 * db), 10.0 + y_offset);
        cr.line_to(x_offset + (114.0 * db), 0.0 + y_offset);
        cr.stroke().unwrap();

        cr.move_to(x_offset + (18.0 * db) - 3.0, 20.0 + y_offset);
        let _ = cr.show_text("3");
        cr.move_to(x_offset + (36.0 * db) - 3.0, 20.0 + y_offset);
        let _ = cr.show_text("6");
        cr.move_to(x_offset + (54.0 * db) - 3.0, 20.0 + y_offset);
        let _ = cr.show_text("9");
        cr.move_to(x_offset + (74.0 * db) - 9.0, 20.0 + y_offset);
        let _ = cr.show_text("+20");
        cr.move_to(x_offset + (94.0 * db) - 9.0, 20.0 + y_offset);
        let _ = cr.show_text("+40");
        cr.move_to(x_offset + (114.0 * db) - 9.0, 20.0 + y_offset);
        let _ = cr.show_text("+60");
    }

    pub fn update_tx(&mut self, radio_mutex: &RadioMutex) {
        let cr =
            Context::new(self.surface.clone()).expect("Couldn't create cairo context from surface");
        let mut r = radio_mutex.radio.lock().unwrap();

        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.paint().unwrap();

        // calculate the SWR
        let fwd_power = r.alex_forward_power as f32;
        let rev_power = r.alex_reverse_power as f32;

        // temp only ORION2 constants
        let c1 = 5.0;
        let c2 = 0.108;
        let v_fwd = (fwd_power / 4095.0) * c1;
        let fwd = (v_fwd * v_fwd) / c2;

        let v_rev = (rev_power / 4095.0) * c1;
        let rev = (v_rev * v_rev) / c2;

        let mut this_swr = (1.0 + (rev / fwd).sqrt()) / (1.0 - (rev / fwd).sqrt());
        if this_swr < 0.0 {
            this_swr = 1.0;
        }

        if !this_swr.is_nan() && !this_swr.is_infinite() {
            let alpha = 0.7;
            r.swr = (alpha * this_swr) + ((1.0 - alpha) * r.swr);

            let fwd_text = format!("FWD: {}", fwd);
            let rev_text = format!("REV: {}", rev);
            let swr_text = format!("SWR: {}:1.0", r.swr);

            cr.set_source_rgb(0.0, 0.0, 0.0);
            cr.move_to(5.0, 10.0);
            let _ = cr.show_text(&fwd_text);
            cr.move_to(5.0, 20.0);
            let _ = cr.show_text(&rev_text);
            cr.move_to(5.0, 30.0);
            let _ = cr.show_text(&swr_text);
        }
    }

    pub fn draw(&self, cr: &Context) {
        cr.set_source_surface(&self.surface, 0.0, 0.0)
            .expect("failed to set source surface");
        cr.paint().expect("Failed to pant surface");
    }
}
