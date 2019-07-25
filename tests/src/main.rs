extern crate portaudio;

use portaudio as pa;
use audio_engine::audio_objects::{
    AudioComponent,
    NaiveTableOsc,
    TanHWaveshaper
};
use audio_engine::tables::{SINE_2048, TRIANGLE_2};
const CHANNELS: i32 = 2;
const NUM_SECONDS: i32 = 5;
const SAMPLE_RATE: f64 = 44_100.0;
const FRAMES_PER_BUFFER: u32 = 64;

fn main() {
    match run() {
        Ok(_) => {}
        e => {
            eprintln!("Example failed with the following: {:?}", e);
        }
    }
}

fn run() -> Result<(), pa::Error> {
    println!(
        "PortAudio Test: output sine wave. SR = {}, BufSize = {}",
        SAMPLE_RATE, FRAMES_PER_BUFFER
    );

    let mut modmod = NaiveTableOsc::new(&SINE_2048);
    let mut modulator = NaiveTableOsc::new(&SINE_2048);
    let mut sine_osc = NaiveTableOsc::new(&SINE_2048);
    let mut disto_mod = NaiveTableOsc::new(&TRIANGLE_2);
    let mut output = TanHWaveshaper::new();

    let mut state = [
        modmod.initial_state(),
        modulator.initial_state(),
        sine_osc.initial_state(),
        disto_mod.initial_state(),
        output.initial_state()
    ];

    let pa = pa::PortAudio::new()?;
    let settings =
        pa.default_output_stream_settings(CHANNELS, SAMPLE_RATE, FRAMES_PER_BUFFER)?;


    let callback = move |pa::OutputStreamCallbackArgs { buffer, frames, .. }| {
        let mut idx = 0;
        for _ in 0..frames {
            let samp = state[4];

            let old_state = state.clone();

            state[0] = modmod.step_and_sample((0.3, 300.0, 660.0));
            state[1] = modulator.step_and_sample((old_state[0], 220.0, 440.0));
            state[2] = sine_osc.step_and_sample((old_state[1], 1.0, 0.0));
            state[3] = disto_mod.step_and_sample((2.3, 3.0, 3.2));
            state[4] = output.step_and_sample((old_state[2], old_state[3]));

            buffer[idx] = samp;
            buffer[idx + 1] = samp;
            idx += 2;
        }
        pa::Continue
    };

    let mut stream = pa.open_non_blocking_stream(settings, callback)?;
   // let mut stream = pa.open_blocking_stream(settings);

    stream.start()?;

    println!("Play for {} seconds.", NUM_SECONDS);
    pa.sleep(NUM_SECONDS * 1_000);

    stream.stop()?;
    stream.close()?;

    println!("Test finished.");

    Ok(())
}
