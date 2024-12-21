use std::{
    thread,
    time::{Duration, Instant},
};

use cpal::{traits::DeviceTrait, Device};
use crossbeam_channel::{Receiver, Sender, TryRecvError};
use log::{info, warn};
use serialport::{SerialPort, SerialPortInfo, SerialPortType};

use crate::{
    app::FromFrontend,
    audio::{self, Signal, SystemMessage},
    utils,
};

pub enum DmxUniverse {
    Dummy,
    Real(DmxUniverseReal),
}

impl DmxUniverse {
    pub fn new(port_path: String) -> Self {
        Self::Real(DmxUniverseReal::new(port_path))
    }

    pub fn new_dummy() -> Self {
        Self::Dummy
    }

    pub fn signal(&mut self, signal: Signal) {
        match self {
            DmxUniverse::Dummy => {}
            DmxUniverse::Real(dmx_universe_real) => dmx_universe_real.signal(signal),
        }
    }
}

struct DmxUniverseReal {
    serial: Box<dyn SerialPort>,
    channels: [u8; 513],
    last_update: Instant,
}

impl DmxUniverseReal {
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
            last_update: Instant::now(),
        }
    }

    fn signal(&mut self, signal: Signal) {
        match signal {
            Signal::BeatVolume(volume) => {
                // TODO: engine here
                // return;
                if volume >= 1 {
                    println!("V={volume}");
                    // self.channels[1] = volume / 10;
                    self.channels[1] = 255;
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
            Signal::Bass(v) => {
                if v > 20 && self.last_update.elapsed().as_millis() > 100 {
                    const CHANNEL_OFFSET_STROBE: usize = 10;

                    self.last_update = Instant::now();
                    self.channels[1 + CHANNEL_OFFSET_STROBE - 1] = 255;
                    self.channels[2 + CHANNEL_OFFSET_STROBE - 1] = 255;
                    self.channels[3 + CHANNEL_OFFSET_STROBE - 1] = 255;
                    self.channels[4 + CHANNEL_OFFSET_STROBE - 1] = 255;
                    self.write_to_serial();
                    spin_sleep::sleep(Duration::from_millis(1));

                    self.channels[1 + CHANNEL_OFFSET_STROBE -1] = 0;

                    self.write_to_serial();
                }
            }
            Signal::Volume(v) => {
                if v < 10 {
                    self.channels[1] = 30;
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

pub struct UsbDevice {
    pub vid: u16,
    pub pid: u16,
}

pub const EUROLITE_USB_DMX512_PRO_CABLE_INTERFACE: UsbDevice = UsbDevice {
    vid: 1027,
    pid: 24577,
};

pub const USB_DEVICES: [UsbDevice; 1] = [EUROLITE_USB_DMX512_PRO_CABLE_INTERFACE];
// const SERIAL_ERROR_RETRY: Duration = Duration::from_secs(5);

pub enum DMXControl {
    ChangePort(Option<SerialPortInfo>),
}

// pub fn _dmx_thread(
//     control_receiver: Receiver<DMXControl>,
//     signal_receiver: Receiver<Signal>,
//     system_out: Sender<SystemMessage>,
// ) {
//     let ports = serialport::available_ports().unwrap();
//
//     // Update available ports to frontend.
//     system_out
//         .send(SystemMessage::SerialDevicesView(ports.clone()))
//         .unwrap();
//
//     println!("{ports:?}");
//
//     let mut port = ports.into_iter().find(|p| {
//         let SerialPortType::UsbPort(usb) = p.port_type.clone() else {
//             return false;
//         };
//
//         USB_DEVICES
//             .iter()
//             .any(|d| d.pid == usb.pid && d.vid == usb.vid)
//     });
//
//     if port.is_none() {
//         warn!("No default DMX serial output available");
//     }
//
//     // let Some(port) = port.cloned() else {
//     //     warn!("[DMX] No default serial device available...");
//     //     system_out
//     //         .send(SystemMessage::SerialSelected(None))
//     //         .unwrap();
//     // };
//
//     // let mut port = Some(port);
//
//     loop {
//         let mut universe = None;
//
//         if let Some(port) = port {
//             let name = port.port_name.clone();
//             info!("[DMX] Using serial device: {name}");
//             system_out
//                 .send(SystemMessage::SerialSelected(Some(port.clone())))
//                 .unwrap();
//             universe = Some(DmxUniverse::new(name));
//         }
//
//         'inner: loop {
//             // Dispatch signals to frontend and to DMX engine.
//             match signal_receiver.try_recv() {
//                 Ok(signal) => {
//                     if let Some(ref mut universe) = universe {
//                         universe.signal(signal);
//                     }
//                 }
//                 Err(TryRecvError::Empty) => {}
//                 Err(err) => panic!("{err:?}"),
//             }
//
//             match control_receiver.try_recv() {
//                 Ok(DMXControl::ChangePort(new_port)) => {
//                     println!("select port: {new_port:?}");
//                     port = new_port;
//                     break 'inner;
//                 }
//                 Err(TryRecvError::Empty) => {}
//                 Err(err) => panic!("{err:?}"),
//             }
//
//             // match system_receiver.try_recv() {
//             //     Ok(SystemMessage::LoopSpeed(speed)) => todo!(""),
//             //     // .emit("msg", ToFrontend::Speed(speed.as_micros() as usize)),
//             //     // .unwrap(),
//             //     Err(TryRecvError::Empty) => {}
//             //     Err(err) => panic!("{err:?}"),
//             // }
//         }
//     }
// }

pub fn audio_thread(
    from_frontend: Receiver<FromFrontend>,
    signal_out_0: Sender<Signal>,
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

            let selected_device = devices
                .iter()
                .find(|dev| dev.1.name().unwrap().contains("CABLE Output"))
                .unwrap_or_else(|| &devices[0]);

            let host = selected_device.0.name().to_string();
            let device_name = selected_device.1.name().unwrap();

            println!(
                "{}",
                devices
                    .iter()
                    .map(|d| d.1.name().unwrap())
                    .collect::<Vec<String>>()
                    .join("|")
            );
            println!("Selected default audio device: {host} | {device_name}");

            device = Some(utils::device_from_names(host, device_name).unwrap());

            device_changed = true;
        } else if device_changed {
            let (sig_0, sys) = (signal_out_0.clone(), system_out.clone());
            {
                let device = device.clone();
                thread::spawn(move || audio::foo(device.unwrap(), sig_0, sys));
            }

            device_changed = false;
            println!(
                "Started audio detector thread: {}...",
                device.clone().unwrap().name().unwrap()
            );
        }
    }
}
