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
        if width!=0 && height !=0 {
            let new_pixbuf = Pixbuf::new(Colorspace::Rgb, false, 8, width, height).unwrap();
            self.pixbuf = new_pixbuf;
        }
    }

    pub fn update(&mut self, width:i32, height: i32, radio_mutex: &RadioMutex, new_pixels: &Vec<f32>) {
        let low_R = 0.0; // low is black
        let low_G = 0.0;
        let low_B = 0.0;
        let mid_R = 1.0; // low is red
        let mid_G = 0.0;
        let mid_B = 0.0;
        let high_R = 1.0; // low is yellow
        let high_G = 1.0;
        let high_B = 0.0;

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
            let mut R = 0.0;
            let mut G = 0.0;
            let mut B = 0.0;
            let mut max_percent = 0.0;
            for x in 0..waterfall_width {
                let mut value: f32 = new_pixels[x as usize + pan as usize] as f32;
                average += value;
                if value <= r.band_info[b].waterfall_low {
                    R = 0.0;
                    G = 0.0;
                    B = 0.0;
                } else if value >= r.band_info[b].waterfall_high {
                    R = 240.0;
                    G = 240.0;
                    B = 240.0;
                } else {
                    let range = r.band_info[b].waterfall_high - r.band_info[b].waterfall_low;
                    let offset = value - r.band_info[b].waterfall_low;
                    let mut percent = 100.0 * offset / range;
                    if percent > max_percent {
                        max_percent = percent;
                    }

                    if percent < 10.0 {
                        R = 0.0;
                        G = R;
                        B = 64.0;
                    } else if percent < 20.0 {
                        R = 128.0;
                        G = R;
                        B = 0.0;
                    } else if percent < 40.0 {
                        R = 255.0;
                        G = R;
                        B = 0.0;
                    } else if percent < 40.0 {
                        R = 64.0;
                        G = 0.0;
                        B = 0.0;
                    } else if percent < 60.0 {
                        R = 128.0;
                        G = 0.0;
                        B = 0.0;
                    } else if percent < 80.0 {
                        R = 192.0;
                        G = 0.0;
                        B = 0.0;
                    } else {
                        R = 255.0;
                        G = 0.0;
                        B = 0.0;
                    }
                }
                let ix = (x * 3) as usize;
                pixels[ix] = R as u8;
                pixels[ix + 1] = G as u8;
                pixels[ix + 2] = B as u8;
            }
            //println!("average {} max_percent {}", average / width as f32, max_percent);
            if r.waterfall_auto {
                r.band_info[b].waterfall_low = (r.band_info[b].waterfall_low + (average / width as f32)) / 2.0;
            }
        } // unsafe
    }

    pub fn draw(&self, cr: &Context, width: i32, height: i32) {
                cr.set_source_pixbuf(&self.pixbuf, 0.0, 0.0);
                cr.paint().unwrap();
    }
}
