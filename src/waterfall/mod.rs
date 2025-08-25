use gtk::prelude::*;
use gtk::cairo::Context;
use gdk_pixbuf::{Colorspace, Pixbuf};

use crate::radio::RadioMutex;

#[derive(Clone)]
pub struct Waterfall {
    rx: usize,
    pixbuf: Pixbuf,
}

impl Waterfall {

    pub fn new(id: usize, width: i32, height: i32) -> Self {
        let rx = id;
        let pixbuf = Pixbuf::new(Colorspace::Rgb, false, 8, width, height).unwrap();
        Self {
            rx,
            pixbuf,
        }
    }

    pub fn resize(&mut self, width: i32, height: i32) {
        let new_pixbuf = Pixbuf::new(Colorspace::Rgb, false, 8, width, height).unwrap();
        self.pixbuf = new_pixbuf;
    }

    pub fn update(&mut self, width:i32, height: i32, radio_mutex: &RadioMutex, new_pixels: &Vec<f32>) {
        let mut r = radio_mutex.radio.lock().unwrap();
        let mut average = 0.0;
        unsafe {
            let pixels = self.pixbuf.pixels();
            let width = self.pixbuf.width() as usize;
            let height = self.pixbuf.height() as usize;
            let rowstride = self.pixbuf.rowstride() as usize;
            let channels = self.pixbuf.n_channels() as usize;

            for y in (0..height - 1).rev() { // Iterate in reverse order
                let src_offset = (y * rowstride) as usize;
                let dest_offset = ((y + 1) * rowstride) as usize;
                if dest_offset + rowstride <= pixels.len() {
                    pixels.copy_within(src_offset..src_offset + rowstride as usize, dest_offset);
                }
            }

            // fill in the top line with the latest spectrum data
            let waterfall_width = r.receiver[self.rx].spectrum_width;
            let pan = ((new_pixels.len() as f32 - waterfall_width as f32) / 100.0) * r.receiver[self.rx].pan as f32;

            let b = r.receiver[self.rx].band.to_usize();
            for x in 0..waterfall_width {
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
                r.band_info[b].waterfall_low = (average / waterfall_width as f32) + r.waterfall_calibrate;
            }
        } // unsafe
    }

    pub fn draw(&self, cr: &Context, width: i32, height: i32) {
                cr.set_source_pixbuf(&self.pixbuf, 0.0, 0.0);
                cr.paint().unwrap();
    }
}
