#![allow(unused)]

use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{Device, HostId};
use log::LevelFilter;
use std::io::{Read, Write};
use std::process::exit;

pub fn init_logger() {
    simple_logger::SimpleLogger::new()
        .with_level(LevelFilter::Debug)
        .with_colors(true)
        .with_utc_timestamps()
        .init()
        .unwrap();
}

pub fn device_from_name(dev_id: String) -> Option<Device> {
    let devices = get_input_devices_flat();

    let selected_device: Option<(HostId, Device)> = devices
        .into_iter()
        .find(|(this_host, this_dev)| this_dev.name().unwrap() == dev_id);

    let Some((_, device)) = selected_device else {
        return None;
    };

    Some(device)
}

pub fn device_from_names(host_id: String, dev_id: String) -> Option<Device> {
    let devices = get_input_devices_flat();

    let selected_device: Option<(HostId, Device)> =
        devices.into_iter().find(|(this_host, this_dev)| {
            this_host.name() == host_id && this_dev.name().unwrap() == dev_id
        });

    let Some((_, device)) = selected_device else {
        return None;
    };

    Some(device)
}

/// Returns all valid and available input devices.
fn get_input_devices() -> Vec<(cpal::HostId, Vec<cpal::Device>)> {
    cpal::available_hosts()
        .into_iter()
        .map(|host_id| {
            let host = cpal::host_from_id(host_id).expect("should know the just queried host");
            (host_id, host)
        })
        .map(|(host_id, host)| (host_id, host.devices()))
        .filter(|(_, devices)| devices.is_ok())
        .map(|(host_id, devices)| (host_id, devices.unwrap()))
        .map(|(host_id, devices)| {
            (
                host_id,
                devices
                    .into_iter()
                    // check: is input device?
                    .filter(|dev| dev.default_input_config().is_ok())
                    // check: can we get its name?
                    .filter(|dev| dev.name().is_ok())
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>()
}

pub fn get_input_devices_flat() -> Vec<(cpal::HostId, cpal::Device)> {
    get_input_devices()
        .into_iter()
        .flat_map(|(host_id, devices)| {
            devices
                .into_iter()
                .map(|dev| (host_id, dev))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>()
}

// pub fn select_audio_device() -> cpal::Device {
//     let mut devices = get_input_devices_flat();
//
//     if devices.is_empty() {
//         println!("No audio input device available");
//         exit(0);
//     }
//
//     if devices.len() == 1 {
//         return devices.swap_remove(0).1;
//     }
//
//     println!("Available input devices:");
//     for (device_i, (host_id, device)) in devices.iter().enumerate() {
//         println!(
//             "[{}]: {:?} - {}",
//             device_i,
//             host_id,
//             device
//                 .name()
//                 .expect("should be existent at that point due to the filtering")
//         );
//     }
//
//     print!("Type a number: ");
//     std::io::stdout().flush().unwrap();
//
//     let mut buf = [0];
//     std::io::stdin().read_exact(&mut buf).unwrap();
//     println!(); // newline
//     let buf = std::str::from_utf8(&buf).unwrap();
//     let choice = str::parse::<usize>(buf).unwrap();
//
//     // Remove element and take ownership.
//     devices.swap_remove(choice).1
// }
