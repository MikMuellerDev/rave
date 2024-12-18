use std::{thread, time::Duration};

use cpal::{traits::DeviceTrait, Device};
use crossbeam_channel::{Receiver, Sender, TryRecvError};
use log::{info, warn};
use serialport::{SerialPort, SerialPortType};

use crate::{
    app::FromFrontend,
    audio::{self, Signal, SystemMessage},
    utils,
};

struct DmxUniverse {
    serial: Box<dyn SerialPort>,
    channels: [u8; 513],
}

impl DmxUniverse {
    fn new(port_path: String) -> Self {
        let port = serialport::new(port_path, 250000)
            .timeout(Duration::from_millis(1))
            .stop_bits(serialport::StopBits::Two)
            .data_bits(serialport::DataBits::Eight)
            .parity(serialport::Parity::None)
            .open()
            .expect("Failed to open port");

        Self {
            serial: port,
            channels: [0; 513],
        }
    }

    fn signal(&mut self, signal: Signal) {
        match signal {
            Signal::BeatVolume(volume) => {
                // TODO: engine here
                // return;
                if volume >= 1 {
                    println!("V={volume}");
                    self.channels[1] = volume / 10;
                    self.channels[2] = 255;
                    self.channels[3] = 0;
                    self.channels[4] = 0;
                    self.write_to_serial();
                } else {
                    self.channels[1] = 0;
                    self.write_to_serial();
                }

                // spin_sleep::sleep(Duration::from_millis(10));
            }
            Signal::BeatAlgo(_) => {
                // return;
                self.channels[1] = 255;
                self.channels[2] = 255;
                self.channels[3] = 255;
                self.channels[4] = 255;

                self.write_to_serial();
                spin_sleep::sleep(Duration::from_millis(10));

                self.channels[1] = 0;

                self.write_to_serial();
            }
            Signal::Bass(_) => todo!(),
            Signal::Volume(v) => {
                if v < 10 {
                    self.channels[1] = 255;
                    self.channels[2] = 255;
                    self.channels[3] = 0;
                    self.channels[4] = 255;

                    self.write_to_serial();
                } 
            }
        }
    }

    fn send_break(&self, duration: Duration) {
        self.serial.set_break().expect("Failed to set break");
        spin_sleep::sleep(duration);
        self.serial.clear_break().expect("Failed to clear break");
    }

    fn write_to_serial(&mut self) {
        self.send_break(Duration::from_micros(100));
        spin_sleep::sleep(Duration::from_micros(100));
        self.serial.write_all(&self.channels).unwrap();
        self.serial.flush().unwrap();
    }
}

struct UsbDevice {
    vid: u16,
    pid: u16,
}

const EUROLITE_USB_DMX512_PRO_CABLE_INTERFACE: UsbDevice = UsbDevice {
    vid: 1027,
    pid: 24577,
};

const USB_DEVICES: [UsbDevice; 1] = [EUROLITE_USB_DMX512_PRO_CABLE_INTERFACE];
const SERIAL_ERROR_RETRY: Duration = Duration::from_secs(5);

pub fn dmx_thread(signal_receiver: Receiver<Signal>, system_out: Sender<SystemMessage>) {
    loop {
        let ports = serialport::available_ports().unwrap();

        // Update available ports to frontend.
        system_out
            .send(SystemMessage::SerialDevicesView(ports.clone()))
            .unwrap();

        println!("{ports:?}");

        let port = ports.iter().find(|p| {
            let SerialPortType::UsbPort(usb) = p.port_type.clone() else {
                return false;
            };

            USB_DEVICES
                .iter()
                .any(|d| d.pid == usb.pid && d.vid == usb.vid)
        });

        let Some(port) = port else {
            warn!("[DMX] No default serial device available...");
            system_out
                .send(SystemMessage::SerialSelected(None))
                .unwrap();
            thread::sleep(SERIAL_ERROR_RETRY);
            continue;
        };

        let name = port.port_name.clone();
        info!("[DMX] Found default serial device");
        system_out
            .send(SystemMessage::SerialSelected(Some(port.clone())))
            .unwrap();
        let mut universe = DmxUniverse::new(name);

        loop {
            // Dispatch signals to frontend and to DMX engine.
            match signal_receiver.try_recv() {
                Ok(signal) => {
                    universe.signal(signal);
                }
                Err(TryRecvError::Empty) => {}
                Err(err) => panic!("{err:?}"),
            }

            // match system_receiver.try_recv() {
            //     Ok(SystemMessage::LoopSpeed(speed)) => todo!(""),
            //     // .emit("msg", ToFrontend::Speed(speed.as_micros() as usize)),
            //     // .unwrap(),
            //     Err(TryRecvError::Empty) => {}
            //     Err(err) => panic!("{err:?}"),
            // }
        }
    }
}

pub fn audio_thread(
    from_frontend: Receiver<FromFrontend>,
    signal_out_0: Sender<Signal>,
    signal_out_1: Sender<Signal>,
    system_out: Sender<SystemMessage>,
) {
    // let begin_msg = from_frontend.recv().unwrap();
    println!("[audio] Thread started!");

    // let FromFrontend::NewWindow(window) = begin_msg else {
    //     panic!("Illegal behaviour");
    // };

    // let mut count = 0;
    // let mut increment = true;
    // let step = 25;
    let heartbeat_delay = Duration::from_millis(1000);

    let mut device: Option<Device> = None;
    let mut device_changed = false;

    // From audio to frontend.
    // let (signal_out, signal_receiver) = mpsc::channel();
    // let (system_out, system_receiver) = mpsc::channel();

    // let w = window.clone();

    // TODO: put the DMX thread under main!

    loop {
        thread::sleep(heartbeat_delay);
        // TODO
        // window.emit("msg", ToFrontend::Heartbeat).unwrap();

        match from_frontend.try_recv() {
            // Ok(FromFrontend::NewWindow(_)) => unreachable!(),
            Ok(FromFrontend::SelectInputDevice(dev)) => {
                device = Some(dev.clone());
                device_changed = true;
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                unreachable!("broken")
            }
        };

        if device.is_none() {
            let devices = utils::get_input_devices_flat();
            if devices.is_empty() {
                panic!("No devices");
            }


            let selected_device = devices.iter().find(|dev| dev.1.name().unwrap().contains("CABLE Output")).unwrap();

            let host = selected_device.0.  name().to_string();
            let device_name = selected_device.1.name().unwrap();

            println!("{}", devices.iter().map(|d|d.1.name().unwrap()).collect::<Vec<String>>().join("|"));
            println!("Selected default audio device: {host} | {device_name}");

            device = Some(utils::device_from_names(host, device_name).unwrap());

            device_changed = true;
        } else if device_changed {
            let (sig_0, sig_1, sys) = (signal_out_0.clone(), signal_out_1.clone(), system_out.clone());
            {
            let device = device.clone();
            thread::spawn(move || audio::foo(
                device.unwrap(),
                sig_0,
                sig_1,
                 sys,
                ));
            }

            device_changed = false;
            println!(
                "Started audio detector thread: {}...",
                device.clone().unwrap().name().unwrap()
            );
        }
    }
}
