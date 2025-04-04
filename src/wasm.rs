//! Small example of how to instantiate a wasm module that imports one function,
//! showing how you can fill in host functionality for a wasm module.

// You can execute this example with `cargo run --example hello`

use std::time::{Instant, UNIX_EPOCH};

use wasmtime::*;

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
    fn serialize(&self, timer_start: Instant) -> [i32; 6] {
        [
            Instant::now().duration_since(timer_start).as_millis() as i32, // Time.
            self.volume.into(),
            self.beat_volume.into(),
            self.bass.into(),
            self.bass_avg.into(),
            self.bpm.into(),
        ]
    }
}

pub struct TickEngine {
    timer_start: Instant,
    data: Vec<i32>,
    wasm: Option<WasmEngine>,
}

pub struct WasmEngine {
    store: Store<MyState>,
    instance: Instance,
    memory: Memory,
}

impl TickEngine {
    pub fn create() -> Result<Self> {
        let mut engine = TickEngine {
            timer_start: Instant::now(),
            data: vec![],
            wasm: None,
        };

        engine.init_wasm();

        Ok(engine)
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

        // Our wasm module we'll be instantiating requires one imported function.
        // the function takes no parameters and returns no results. We create a host
        // implementation of that function here, and the `caller` parameter here is
        // used to get access to our original `MyState` value.
        println!("Creating callback...");
        let log_function = Func::wrap(
            &mut store,
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
        );

        // Once we've got that all set up we can then move to the instantiation
        // phase, pairing together a compiled module as well as a set of imports.
        // Note that this is where the wasm `start` function, if any, would run.
        println!("Instantiating module...");
        let imports = [log_function.into()];
        let instance = Instance::new(&mut store, &module, &imports)?;

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
        self.init_wasm()
    }

    pub fn tick(&mut self, input: TickInput) -> Result<Vec<i32>> {
        let wasm = self.wasm.as_mut().unwrap();

        // Get the function
        let func = wasm
            .instance
            .get_typed_func::<(i32, i32, i32, i32, i32, i32), ()>(
                &mut wasm.store,
                "internal_tick",
            )?;

        let tick_array_offset = 0x1000; // Arbitrary offset
        let tick_array_data = input.serialize(self.timer_start);
        let tick_array_len = tick_array_data.len() as i32;

        // Convert to little-endian bytes
        let mut tick_array_bytes = Vec::new();
        for &num in &tick_array_data {
            tick_array_bytes.extend_from_slice(&num.to_le_bytes());
        }

        // DMX array
        // TODO: actually store on self and not create new?
        let dmx_array_offset = 0x2000; // Arbitrary offset
        let dmx_array_data: Vec<u8> = vec![0; 513];
        let dmx_array_len = dmx_array_data.len() as i32;

        // Convert to little-endian bytes
        let mut dmx_array_bytes = Vec::new();
        for &num in &dmx_array_data {
            dmx_array_bytes.extend_from_slice(&num.to_le_bytes());
        }

        // Data array
        let data_array_offset = 0x9000; // Arbitrary offset
        let data_array_len = self.data.len();

        // Convert to little-endian bytes
        let mut data_array_bytes = Vec::new();
        for &num in &self.data {
            data_array_bytes.extend_from_slice(&num.to_le_bytes());
        }

        // Write the array into Wasm memory
        wasm.memory
            .write(&mut wasm.store, dmx_array_offset, &dmx_array_bytes)?;

        wasm.memory
            .write(&mut wasm.store, tick_array_offset, &tick_array_bytes)?;

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
            ),
        )?;

        // Read back the modified DMX + data array
        let mut updated_dmx_bytes = vec![0u8; dmx_array_bytes.len()];
        wasm.memory
            .read(&mut wasm.store, dmx_array_offset, &mut updated_dmx_bytes)?;

        // Convert bytes back to integers
        let updated_array: Vec<i32> = updated_dmx_bytes
            .chunks_exact(4)
            .map(|chunk| i32::from_le_bytes(chunk.try_into().unwrap()))
            .collect();

        // println!("Updated DMX: {:?}", &updated_array[0..10]);

        let mut updated_data_bytes = vec![0u8; data_array_bytes.len()];
        wasm.memory
            .read(&mut wasm.store, data_array_offset, &mut updated_data_bytes)?;

        // Convert bytes back to integers
        let updated_data_array: Vec<i32> = updated_data_bytes
            .chunks_exact(4)
            .map(|chunk| i32::from_le_bytes(chunk.try_into().unwrap()))
            .collect();

        // println!("Updated data: {:?}", &updated_array[0..10]);

        /// END
        // let run = instance.get_typed_func::<(TickInput), ()>(&mut store, "tick")?;

        // // And last but not least we can call it!
        // println!("Calling export...");
        // run.call(&mut store, ())?;

        // println!("Done.");
        Ok(updated_array)
    }
}
