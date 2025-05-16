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
use gtk::{Button, Label, ListBox, ListBoxRow, Orientation, Window};
use network_interface::NetworkInterface;
use network_interface::NetworkInterfaceConfig;
use nix::sys::socket::setsockopt;
use nix::sys::socket::sockopt::ReuseAddr;
use nix::sys::socket::sockopt::ReusePort;
use std::net::UdpSocket;
use std::net::SocketAddr;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;


#[derive(Copy, Clone, Debug)]
pub struct Device {
    pub address: SocketAddr,
    pub my_address: SocketAddr,
    pub device: u8,
    pub protocol: u8,
    pub version: u8,
    pub status: u8,
    pub mac: [u8;6],
    pub supported_receivers: u8,
    pub supported_transmitters: u8,
    pub adcs: u8,
    pub frequency_min: u64,
    pub frequency_max: u64,
}


fn add_device(devices: Rc<RefCell<Vec<Device>>>, address: SocketAddr, my_address: SocketAddr, device: u8,protocol: u8,version:u8,status: u8,mac: [u8;6],supported_receivers: u8,supported_transmitters: u8,adcs: u8,frequency_min: u64,frequency_max: u64) {
    devices.borrow_mut().push(Device{address,my_address,device,protocol,version,status,mac,supported_receivers,supported_transmitters,adcs,frequency_min,frequency_max});
}

pub fn protocol1_discovery(devices: Rc<RefCell<Vec<Device>>>, socket_addr: SocketAddr) {
    //let socket = UdpSocket::bind("0.0.0.0:0").expect("bind failed");
    let socket = UdpSocket::bind(socket_addr).expect("bind failed");
    socket.set_broadcast(true).expect("set_broadcast call failed");
    socket.set_read_timeout(Some(Duration::new(1, 0))).expect("set_read_timeout call failed");
    socket.set_write_timeout(Some(Duration::new(1, 0))).expect("set_read_timeout call failed");

    let _res = setsockopt(&socket, ReusePort, &true);
    let _res = setsockopt(&socket, ReuseAddr, &true);

    let mut _discover:[u8; 63] = [0xEF,0xFE,0x02,
                                  0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,
                                  0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,
                                  0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,
                                  0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,
                                  0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,
                                  0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00];
    socket.send_to(&_discover, "255.255.255.255:1024").expect("couldn't send data");

    loop {

        // Receives a single datagram message on the socket. If `buf` is too small to hold
        // the message, it will be cut off.
        let mut buf = [0; 1024];
        match socket.recv_from(&mut buf) {
            Ok((amt,src)) => {
                let local_addr = socket.local_addr().expect("failed to get local address");
                if amt == 60 {
                    let mac: [u8;6] = [buf[3],buf[4],buf[5],buf[6],buf[7],buf[8]];
                    let mut adcs=1;
                    let mut supported_receivers=1;
                    let mut supported_transmitters=1;
                    let mut frequency_min=0;
                    let mut frequency_max=61440000;

                    match  buf[11] {
                        0=>{ // METIS
                           adcs=1;
                           supported_receivers=5;
                           supported_transmitters=1;
                           frequency_min=0;
                           frequency_max=61440000;
                           },
                        1=>{ // HERMES
                           adcs=1;
                           supported_receivers=5;
                           supported_transmitters=1;
                           frequency_min=0;
                           frequency_max=61440000;
                           },
                        4=>{ // ANGELIA
                           adcs=2;
                           supported_receivers=7;
                           supported_transmitters=1;
                           frequency_min=0;
                           frequency_max=61440000;
                           },
                        5=>{ // ORION
                           adcs=2;
                           supported_receivers=7;
                           supported_transmitters=1;
                           frequency_min=0;
                           frequency_max=61440000;
                           },
                        6=>{ // HERMES_LITE
                           if buf[9] < 42 {
                               // HERMES_LITE V1
                               supported_receivers=2;
                           } else {
                               // HERMES_LITE V2
                               supported_receivers=buf[19];
                           }
                           adcs=1;
                           supported_transmitters=1;
                           frequency_min=0;
                           frequency_max=30720000;
                           },
                        10=>{ // ORION2
                           adcs=2;
                           supported_receivers=7;
                           supported_transmitters=1;
                           frequency_min=0;
                           frequency_max=61440000;
                           },
                        _=>{ // UNKNOWN - use defaults
                           },
                    }
                    add_device(Rc::clone(&devices),src,local_addr,buf[10],1,buf[9],buf[2],mac,supported_receivers,supported_transmitters,adcs,frequency_min,frequency_max);
                } else {
                }
            },
            Err(_e) => {
                break;
            }
        }
    }
    drop(socket);
}

pub fn protocol2_discovery(devices: Rc<RefCell<Vec<Device>>>, socket_addr: SocketAddr) {
    //let socket = UdpSocket::bind("0.0.0.0:0").expect("bind failed");
    let socket = UdpSocket::bind(socket_addr).expect("bind failed");
    socket.set_broadcast(true).expect("set_broadcast call failed");
    socket.set_read_timeout(Some(Duration::new(1, 0))).expect("set_read_timeout call failed");
    socket.set_write_timeout(Some(Duration::new(1, 0))).expect("set_read_timeout call failed");
    let _res = setsockopt(&socket, ReusePort, &true);
    let _res = setsockopt(&socket, ReuseAddr, &true);


    let mut _discover:[u8; 60] = [0x00,0x00,0x00,0x00,0x02,0x00,0x00,0x00,0x00,0x00,
                                0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,
                                0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,
                                0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,
                                0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,
                                0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00];

    socket.send_to(&_discover, "255.255.255.255:1024").expect("couldn't send data");

    loop {

        // Receives a single datagram message on the socket. If `buf` is too small to hold
        // the message, it will be cut off.
        let mut buf = [0; 1024];
        match socket.recv_from(&mut buf) {
            Ok((amt,src)) => {
                if amt == 60 {
                    let mac: [u8;6] = [buf[5],buf[6],buf[7],buf[8],buf[9],buf[10]];
                    let mut adcs=1;
                    let mut supported_receivers=1;
                    let mut supported_transmitters=1;
                    let mut frequency_min=0;
                    let mut frequency_max=61440000;

                    match  buf[11] {
                        0=>{ // ATLAS
                            adcs=1;
                            supported_receivers=5;
                            supported_transmitters=1;
                            frequency_min=0;
                            frequency_max=61440000;
                           },
                        1=>{ // HERMES
                            adcs=1;
                            supported_receivers=5;
                            supported_transmitters=1;
                            frequency_min=0;
                            frequency_max=61440000;
                           },
                        2=>{ // HERMES2
                            adcs=1;
                            supported_receivers=5;
                            supported_transmitters=1;
                            frequency_min=0;
                            frequency_max=61440000;
                           },
                        3=>{ // ANGELIA
                            adcs=2;
                            supported_receivers=7;
                            supported_transmitters=1;
                            frequency_min=0;
                            frequency_max=61440000;
                           },
                        4=>{ // ORION
                            adcs=2;
                            supported_receivers=7;
                            supported_transmitters=1;
                            frequency_min=0;
                            frequency_max=61440000;
                           },
                        5=>{ // ORION2
                            adcs=2;
                            supported_receivers=7;
                            supported_transmitters=1;
                            frequency_min=0;
                            frequency_max=61440000;
                           },
                        6=>{ // HERMES_LITE
                            adcs=1;
                            supported_receivers=5;
                            supported_transmitters=1;
                            frequency_min=0;
                            frequency_max=30720000;
                           },
                        _=>{ // UNKNOWN - use defaults
                           },
                    }
                    let local_addr = socket.local_addr().expect("failed to get local address");
                    add_device(Rc::clone(&devices),src,local_addr,buf[11],2,buf[13],buf[4],mac,supported_receivers,supported_transmitters,adcs,frequency_min,frequency_max);
                } else {
                    println!("Expected 60 bytes but Received: {:?} From {:?}",amt,src);
                }
            },
            Err(_e) => {
                break;
            }
        }
    }
    drop(socket);
}

pub fn discover(devices: Rc<RefCell<Vec<Device>>>) {
    devices.borrow_mut().clear();
    let network_interfaces = NetworkInterface::show().unwrap();
    for itf in network_interfaces.iter() {
        if itf.addr.len()>0 {
            let std::net::IpAddr::V4(ip_addr) = itf.addr[0].ip() else { todo!() };
            let socket_address = SocketAddr::new(std::net::IpAddr::V4(ip_addr),0);
            protocol1_discovery(Rc::clone(&devices), socket_address);
            protocol2_discovery(Rc::clone(&devices), socket_address);
        }
    }
}

pub fn create_discovery_dialog(parent: Option<&impl IsA<gtk::Window>>, discovery_data: Rc<RefCell<Vec<Device>>>, selected_index: Rc<RefCell<Option<i32>>>) -> Window {

    let window = Window::builder()
        .title("rustyHPSDR Discovery")
        .modal(true)
        .transient_for(parent.expect("REASON"))
        .destroy_with_parent(true)
        .default_width(800)
        .default_height(200)
        .build();



    let list = ListBox::builder()
            .margin_top(5)
            .margin_end(5)
            .margin_bottom(5)
            .margin_start(5)
            .build();

    list.set_vexpand(true);

    let discovery_data_clone = Rc::clone(&discovery_data);
    discover(discovery_data_clone);
    populate_list_box(&list.clone(), Rc::clone(&discovery_data));
/*
    let header = create_discovery_row(&["Device", "IP", "MAC", "Protocol", "Version", "Status"], true);
    header.set_sensitive(false); // Disable selection
    list.append(&header);

    let discovery_iter = discovery_data.borrow().clone().into_iter();
    let mut radio = "Unknown";
    for val in discovery_iter {
        match val.protocol {
          1=>match val.device {
                 0=>radio = "METIS",
                 1=>radio = "HERMES",
                 2=>radio = "GRIFFIN",
                 4=>radio = "ANGELIA",
                 5=>radio = "ORION",
                 6=> {
                     if val.version < 42 {
                         radio = "HERMES LITE 1";
                     } else {
                         radio = "HERMES LITE 2";
                     }
                     },
                 10=>radio = "ORION2",
                 _=>radio = "Unknown Radio",
             },
          2=>match val.device {
                 0=>radio = "ATLAS",
                 1=>radio = "HERMES",
                 2=>radio = "HERMES2",
                 3=>radio = "ANGELIA",
                 4=>radio = "ORION",
                 5=>radio = "ORION2",
                 6=>radio = "HERMES_LITE",
                 _=>radio = "Unknown Radio",
             },

          _=>radio = "Unknown Protocol",
        }

        let ip=format!("{}",val.address);
        let mac=format!("{:02X?}",val.mac);
        let protocol=format!("{}",val.protocol);
        let version=format!("{}.{}",val.version/10,val.version%10);
        let mut status = "None";
        if val.status == 2 {
            status = "Available";
        } else if val.status == 3 {
            status = "In Use";
        } else {
            status = "Unknown";
        }

        let row = create_discovery_row(&[radio, &ip, &mac, &protocol, &version, &status], false);
        list.append(&row);
    }
*/

    if discovery_data.borrow().len() > 0 {
        if let Some(first_radio_row) = list.row_at_index(1) {
            first_radio_row.activate();
        }
    }

    let button_box = gtk::Box::new(Orientation::Horizontal, 5);
    button_box.set_halign(gtk::Align::End);

    let rediscover_button = Button::builder().label("Rediscover").build();
    let cancel_button = Button::builder().label("Cancel").build();
    let start_button = Button::builder().label("Start").build();

    button_box.append(&rediscover_button);
    button_box.append(&cancel_button);
    button_box.append(&start_button);

    let main_vbox = gtk::Box::new(Orientation::Vertical, 0);
    main_vbox.append(&list);
    main_vbox.append(&button_box);
    window.set_child(Some(&main_vbox));

    let window_clone = window.clone();
    let selected_index_for_start = selected_index.clone();
    let list_clone = list.clone();
    start_button.connect_clicked(move |_| {
        if let Some(selected_row) = list_clone.selected_row() {
            let index = selected_row.index();
            *selected_index_for_start.borrow_mut() = Some(index);
        } else {
            *selected_index_for_start.borrow_mut() = None;
        }
        window_clone.close();
    });

    let window_clone = window.clone();
    cancel_button.connect_clicked(move |_| {
        window_clone.close();
    });

    let window_clone = window.clone();
    let selected_index_for_rediscover = selected_index.clone();
    //let mut devices_for_rediscover = devices.clone();
    let discovery_data_clone = Rc::clone(&discovery_data);
    let list_clone = list.clone();
    rediscover_button.connect_clicked(move |_| {
        let discovery_data_clone_clone = Rc::clone(&discovery_data_clone);
        discover(discovery_data_clone_clone);
        populate_list_box(&list_clone.clone(), Rc::clone(&discovery_data_clone));
    });

    window
}

fn populate_list_box(list: &ListBox, discovery_data: Rc<RefCell<Vec<Device>>>) {

    // Remove any existing rows
    while let Some(row) = list.row_at_index(0) {
        list.remove(&row);
    }

    // add the Devices
    let header = create_discovery_row(&["Device", "IP", "MAC", "Protocol", "Version", "Status"], true);
    header.set_sensitive(false); // Disable selection
    list.append(&header);

    let discovery_iter = discovery_data.borrow().clone().into_iter();
    let mut radio = "Unknown";
    for val in discovery_iter {
        match val.protocol {
          1=>match val.device {
                 0=>radio = "METIS",
                 1=>radio = "HERMES",
                 2=>radio = "GRIFFIN",
                 4=>radio = "ANGELIA",
                 5=>radio = "ORION",
                 6=> {
                     if val.version < 42 {
                         radio = "HERMES LITE 1";
                     } else {
                         radio = "HERMES LITE 2";
                     }
                     },
                 10=>radio = "ORION2",
                 _=>radio = "Unknown Radio",
             },
          2=>match val.device {
                 0=>radio = "ATLAS",
                 1=>radio = "HERMES",
                 2=>radio = "HERMES2",
                 3=>radio = "ANGELIA",
                 4=>radio = "ORION",
                 5=>radio = "ORION2",
                 6=>radio = "HERMES_LITE",
                 _=>radio = "Unknown Radio",
             },

          _=>radio = "Unknown Protocol",
        }

        let ip=format!("{}",val.address);
        let mac=format!("{:02X?}",val.mac);
        let protocol=format!("{}",val.protocol);
        let version=format!("{}.{}",val.version/10,val.version%10);
        let mut status = "None";
        if val.status == 2 {
            status = "Available";
        } else if val.status == 3 {
            status = "In Use";
        } else {
            status = "Unknown";
        }

        let row = create_discovery_row(&[radio, &ip, &mac, &protocol, &version, &status], false);
        list.append(&row);
    }
}

fn create_discovery_row(columns: &[&str], is_header: bool) -> ListBoxRow {
    let row = ListBoxRow::new();
    let hbox = gtk::Box::new(Orientation::Horizontal, 10);

    let mut col = 0;
    for text in columns {
        let label = Label::new(Some(text));
        label.set_xalign(0.0); // Align text to the left
        if is_header {
            label.add_css_class("heading");
        }
        if col==0 {
            label.set_size_request(100,-1);
        } else if col==1 || col==2 {
            label.set_size_request(150,-1);
        } else {
            label.set_size_request(70,-1);
        }
        hbox.append(&label);
        col += 1;
    }

    row.set_child(Some(&hbox));
    row
}
