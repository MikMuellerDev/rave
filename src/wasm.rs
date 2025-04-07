//! Small example of how to instantiate a wasm module that imports one function,
//! showing how you can fill in host functionality for a wasm module.

// You can execute this example with `cargo run --example hello`

use crossbeam_channel::Sender;
use std::time::{Instant, UNIX_EPOCH};

use actix_web::rt::net::UdpSocket;
use itertools::Itertools;
use wasmtime::*;

use crate::midi;

struct MyState {
    name: String,
    count: usize,
}

#[derive(Clone, Copy)]
pub struct TickInput {
    pub volume: u8,
    pub beat_volume: u8,
    pub bass: u8,
    pub bass_avg: u8,
    pub bpm: u8,
}

impl TickInput {
    fn serialize(&self, timer_start: Instant, initial: bool) -> [i32; 7] {
        [
            Instant::now().duration_since(timer_start).as_millis() as i32, // Time.
            self.volume.into(),
            self.beat_volume.into(),
            self.bass.into(),
            self.bass_avg.into(),
            self.bpm.into(),
            initial as i32,
        ]
    }
}

pub struct TickEngine {
    timer_start: Instant,
    data: Vec<i32>,
    dmx: Vec<u8>,
    wasm: Option<WasmEngine>,
    midi_out: Sender<(u8, u8, u8)>,
}

pub struct WasmEngine {
    store: Store<MyState>,
    instance: Instance,
    memory: Memory,
    // midi_out: SyncSender<(u8, u8, u8)>,
}

impl TickEngine {
    pub fn create(midi_out: Sender<(u8, u8, u8)>) -> Result<Self> {
        let mut engine = TickEngine {
            timer_start: Instant::now(),
            data: vec![0; 1000],
            dmx: vec![0; 513],
            wasm: None,
            midi_out,
        };

        engine.init_wasm()?;
        engine.first_tick()?;

        Ok(engine)
    }

    pub fn dmx(&self) -> &[u8] {
        &self.dmx
    }

    fn init_wasm(&mut self) -> Result<()> {
        // First the wasm module needs to be compiled. This is done with a global
        // "compilation environment" within an `Engine`. Note that engines can be
        // further configured through `Config` if desired instead of using the
        // default like this is here.
        println!("Compiling module...");
        let mut config = Config::new();
        config.strategy(wasmtime::Strategy::Cranelift);
        config.cranelift_opt_level(wasmtime::OptLevel::Speed); // Use `SpeedAndSize` for balance

        let engine = Engine::new(&config)?;
        let module = Module::from_file(&engine, "./wasm/output.wasm")?;

        // After a module is compiled we create a `Store` which will contain
        // instantiated modules and other items like host functions. A Store
        // contains an arbitrary piece of host information, and we use `MyState`
        // here.
        println!("Initializing...");
        let mut store = Store::new(
            &engine,
            MyState {
                name: "hello, world!".to_string(),
                count: 0,
            },
        );

        let mut linker = Linker::new(&engine);

        // Our wasm module we'll be instantiating requires one imported function.
        // the function takes no parameters and returns no results. We create a host
        // implementation of that function here, and the `caller` parameter here is
        // used to get access to our original `MyState` value.
        println!("Creating callback...");
        linker
            .func_wrap(
                "blaulicht",
                "log",
                |mut caller: Caller<'_, MyState>, str_pointer: i32, str_len: i32| {
                    // println!("TRIGGERED CALLBACK");
                    // println!("> {}", caller.data().name);
                    // caller.data_mut().count += 1;

                    let memory = caller
                        .get_export("memory")
                        .and_then(|export| export.into_memory())
                        .expect("Failed to find memory");

                    // Read `len` bytes from memory starting at `ptr`
                    let mut buffer = vec![0u8; str_len as usize];
                    memory
                        .read(&caller, str_pointer as usize, &mut buffer)
                        .expect("Failed to read memory");

                    // Convert bytes to a String
                    let received_string = String::from_utf8_lossy(&buffer).to_string();
                    println!("[WASM] {received_string}");
                },
            )
            .unwrap();
        // let log_function = Func::wrap(
        // );

        let mo = self.midi_out.clone();
        let midi_function = linker
            .func_wrap(
                "blaulicht",
                "midi",
                move |status: i32, kind: i32, value: i32| {
                    // println!("TRIGGERED CALLBACK");
                    // println!("> {}", caller.data().name);
                    // caller.data_mut().count += 1;

                    // println!("[WASM] MIDI: {status}: {kind}: {value}");
                    mo.send((status as u8, kind as u8, value as u8)).unwrap();
                },
            )
            .unwrap();

        // Once we've got that all set up we can then move to the instantiation
        // phase, pairing together a compiled module as well as a set of imports.
        // Note that this is where the wasm `start` function, if any, would run.
        println!("Instantiating module...");
        let instance = linker.instantiate(&mut store, &module)?;

        // Next we poke around a bit to extract the `run` function from the module.
        println!("Extracting export...");

        /// START
        // Get memory reference
        let memory = instance
            .get_memory(&mut store, "memory")
            .expect("Memory not found");

        self.wasm = Some(WasmEngine {
            store,
            instance,
            memory,
        });

        println!("[WASM] initialized.");

        Ok(())
    }

    pub fn reload(&mut self) -> Result<()> {
        self.data.fill(0);
        self.init_wasm()?;
        self.first_tick()
    }

    pub fn first_tick(&mut self) -> Result<()> {
        self.tick(
            TickInput {
                volume: 0,
                beat_volume: 0,
                bass: 0,
                bass_avg: 0,
                bpm: 0,
            },
            &[],
            true,
        )
    }

    pub fn tick(
        &mut self,
        input: TickInput,
        midi_events: &[(u8, u8, u8)],
        initial: bool,
    ) -> Result<()> {
        let wasm = self.wasm.as_mut().unwrap();

        //
        // Tick function.
        //
        let func = wasm
            .instance
            .get_typed_func::<(i32, i32, i32, i32, i32, i32, i32, i32), ()>(
                &mut wasm.store,
                "internal_tick", // TODO: external type and name constants.
            )?;

        //
        // Tick input array.
        //
        let tick_array_offset = 0x1000; // Arbitrary offset
        let tick_array_data = input.serialize(self.timer_start, initial);
        let tick_array_len = tick_array_data.len() as i32;
        let mut tick_array_bytes = Vec::new();
        for &num in &tick_array_data {
            tick_array_bytes.extend_from_slice(&num.to_le_bytes());
        }
        wasm.memory
            .write(&mut wasm.store, tick_array_offset, &tick_array_bytes)?;

        // TODO: macro for this array stuff.

        //
        // MIDI array.
        //
        let midi_array_offset = 0x8000; // TODO: make this offset a const.
        let midi_array_len = midi_events.len() as i32;

        if (midi_array_len > 100) {
            panic!("TOO many MIDI events!");
        }

        let mut midi_array_bytes = Vec::new();

        let midi_events_packed: Vec<u32> = midi_events
            .iter()
            .map(|(a, b, c)| (0u32 | (*a as u32) << 16 | (*b as u32) << 8 | (*c as u32)))
            .collect();

        for &num in &midi_events_packed {
            midi_array_bytes.extend_from_slice(&num.to_le_bytes());
        }
        wasm.memory
            .write(&mut wasm.store, midi_array_offset, &midi_array_bytes)?;

        //
        // DMX array.
        //
        let dmx_array_offset = 0x2000; // TODO: make this offset a const.
        let dmx_array_len = self.dmx.len() as i32;
        let mut dmx_array_bytes = Vec::new();
        for &num in &self.dmx {
            dmx_array_bytes.extend_from_slice(&num.to_le_bytes());
        }
        wasm.memory
            .write(&mut wasm.store, dmx_array_offset, &dmx_array_bytes)?;

        //
        // Data array.
        //
        let data_array_offset = 0x9000; // Arbitrary offset
        let data_array_len = self.data.len();
        let mut data_array_bytes = Vec::new();
        for &num in &self.data {
            data_array_bytes.extend_from_slice(&num.to_le_bytes());
        }
        wasm.memory
            .write(&mut wasm.store, data_array_offset, &data_array_bytes)?;

        // Call the function with the pointer and length
        func.call(
            &mut wasm.store,
            (
                tick_array_offset as i32,
                tick_array_len,
                dmx_array_offset as i32,
                dmx_array_len,
                data_array_offset as i32,
                data_array_len as i32,
                midi_array_offset as i32,
                midi_array_len as i32,
            ),
        )?;

        //
        // Read back the modified DMX array
        //
        let mut updated_dmx_bytes = vec![0u8; dmx_array_bytes.len()];
        wasm.memory
            .read(&mut wasm.store, dmx_array_offset, &mut updated_dmx_bytes)?;
        self.dmx = updated_dmx_bytes;

        //
        // Read back data array.
        //
        let mut updated_data_bytes = vec![0u8; data_array_bytes.len()];
        wasm.memory
            .read(&mut wasm.store, data_array_offset, &mut updated_data_bytes)?;
        let updated_data_bytes: Vec<i32> = updated_data_bytes
            .chunks_exact(4)
            .map(|chunk| i32::from_le_bytes(chunk.try_into().unwrap()))
            .collect();
        self.data = updated_data_bytes;

        Ok(())
    }
}
