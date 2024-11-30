use std::sync::{Arc, Mutex};
use std::thread;
use rodio::{Decoder, OutputStream, Sink};
use std::{fs::File, io::BufReader};

pub struct DMXController {
    universes: Vec<Vec<u8>>,
    port: String,
}

impl DMXController {
    pub fn new(port: String) -> DMXController {
        DMXController {
            universes: vec![vec![0; 512]; 2],
            port,
        }
    }

    pub fn set_value(&mut self, universe: usize, channel: usize, value: u8) {
        self.universes[universe][channel] = value;
        println!("DMX value set: Universe {} Channel {} -> {}", universe, channel, value);
    }
}

pub struct AudioController {
    volume: Arc<Mutex<f32>>, // Volume for all sounds
    player_threads: Vec<(String, Arc<Mutex<bool>>)>, // Store thread handles and sound filenames
}

impl AudioController {
    pub fn new() -> AudioController {
        AudioController {
            volume: Arc::new(Mutex::new(1.0)), // Default volume
            player_threads: Vec::new(),
        }
    }

    // Play sound in a separate thread and store the thread handle
    pub fn play_sound(&mut self, file_name: &str, volume: f32) {
        let file_name_clone = file_name.to_string(); // Clone file_name to move into the thread
        let stop_signal = Arc::new(Mutex::new(false)); // Stop signal for audio thread

        let stop_signal_clone = stop_signal.clone(); // Clone the signal to move into the thread

        let self_volume_clone = Arc::clone(&self.volume); // Clone the volume to move into the thread

        thread::spawn(move || {
            println!("Playing sound: {}", file_name_clone);

            let (_stream, stream_handle) = OutputStream::try_default().unwrap();
            let sink = Sink::try_new(&stream_handle).unwrap();

            // Start playing the audio
            sink.append(Decoder::new(BufReader::new(File::open(&file_name_clone).unwrap())).unwrap());

            // calculate the volume based on the global volume and the sound-specific volume
            let volume = *self_volume_clone.lock().unwrap() * volume;
            sink.set_volume(volume);

            // Check for stop signal periodically (in a loop)
            while !*stop_signal.lock().unwrap() {
                // Sleep briefly so that we can check the stop signal without blocking the entire thread
                thread::sleep(std::time::Duration::from_millis(100));
                sink.set_volume(*self_volume_clone.lock().unwrap() * volume);
            }

            // Stop playback if the signal is set to true
            sink.stop();
            println!("Sound playback stopped");
        });

        // Store the handle and the associated filename
        self.player_threads.push((file_name.to_string(), stop_signal_clone));
    }

    // Stop a specific sound by its file name
    pub fn stop_sound(&mut self, file_name: &str) {
        // Find the thread and set the stop signal to true
        for (name, stop_signal) in self.player_threads.iter_mut() {
            if name == file_name {
                println!("Stopping sound: {}", name);
                *stop_signal.lock().unwrap() = true;
            }
        }
    }

    // Stop all sounds by joining all threads
    pub fn stop_all_sounds(&mut self) {
        for (_, stop_signal) in self.player_threads.iter_mut() {
            *stop_signal.lock().unwrap() = true;
        }
    }

    // Set volume for all sounds
    pub fn set_volume(&mut self, volume: f32) {
        match self.volume.lock() {
            Ok(mut v) => *v = volume,
            Err(e) => {
                // re-call set_volume if the lock fails - most likely something else is holding the lock
                println!("Failed to set volume: {:?}, retrying", e);
            }
        }
        println!("Setting volume to: {}", volume);
    }
}

#[tauri::command]
fn parse_command(
    verb: String,
    args: Vec<String>,
    dmx: tauri::State<'_, Arc<Mutex<DMXController>>>,  // Access DMXController from State
    audio: tauri::State<'_, Arc<Mutex<AudioController>>>, // Access AudioController from State
) -> String {
    println!("Received command: {} with args: {:?}", verb, args);

    match verb.as_str() {
        "set_dmx_value" => {
            if args.len() == 3 {
                let universe = args[0].parse::<usize>().unwrap();
                let channel = args[1].parse::<usize>().unwrap();
                let value = args[2].parse::<u8>().unwrap();

                println!("Setting DMX value: {}:{} to {}", universe, channel, value);
                let mut dmx = dmx.lock().unwrap();
                dmx.set_value(universe, channel, value);
            }
        }
        "play_sound" => {
            if args.len() == 2 {
                let file_name = args[0].clone();
                let volume = args[1].parse::<f32>().unwrap();

                println!("Playing sound: {}", file_name);
                let mut audio = audio.lock().unwrap();
                audio.play_sound(&file_name, volume);
            }
        }
        "stop_sound" => {
            if args.len() == 1 {
                let file_name = args[0].clone();
                println!("Stopping sound: {}", file_name);
                let mut audio = audio.lock().unwrap();
                audio.stop_sound(&file_name);
            }
        }
        "stop_all_sounds" => {
            println!("Stopping all sounds");
            let mut audio = audio.lock().unwrap();
            audio.stop_all_sounds();
        }
        "set_volume" => {
            if args.len() == 1 {
                let volume = args[0].parse::<f32>().unwrap();
                println!("Setting volume to: {}", volume);
                let mut audio = audio.lock().unwrap();
                audio.set_volume(volume);
            }
        }
        _ => {
            println!("Unknown command: {}", verb);
        }
    }

    "done".to_string() // Return response
}

fn main() {
    let dmx = Arc::new(Mutex::new(DMXController::new("/tty/USB0".to_string())));
    let audio = Arc::new(Mutex::new(AudioController::new()));

    tauri::Builder::default()
        .manage(dmx.clone()) // Share the DMXController state
        .manage(audio.clone()) // Share the AudioController state
        .invoke_handler(tauri::generate_handler![parse_command]) // Register the command handler
        .run(tauri::generate_context!());
}
