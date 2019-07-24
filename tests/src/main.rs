extern crate portaudio;

use portaudio as pa;
use audio_engine::audio_objects::Audio;
use audio_engine::audio_objects;
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
    let modmod = audio_objects::NaiveTableOsc::new(0.3, 300, 660, &SINE_2048);
    let modulator = audio_objects::NaiveTableOsc::new(modmod, 220.0, 440.0, &SINE_2048);
    let sine_osc = audio_objects::NaiveTableOsc::new(modulator, 1, 0.0, &SINE_2048);
    let disto_mod = audio_objects::NaiveTableOsc::new(2.3, 3, 3.2, &TRIANGLE_2);
    let mut last = audio_objects::TanHWaveshaper::new(sine_osc, disto_mod);
    // let mut last = audio_objects::NaiveTableOsc::new(440,1, 0, &TRIANGLE_2);
    let pa = pa::PortAudio::new()?;
    let settings =
        pa.default_output_stream_settings(CHANNELS, SAMPLE_RATE, FRAMES_PER_BUFFER)?;

    let callback = move |pa::OutputStreamCallbackArgs { buffer, frames, .. }| {
        let mut idx = 0;
        for _ in 0..frames {
            let samp = last.next();
            buffer[idx] = samp;
            buffer[idx + 1] = samp;
            idx += 2;
        }
        pa::Continue
    };

    let mut stream = pa.open_non_blocking_stream(settings, callback)?;

    stream.start()?;

    println!("Play for {} seconds.", NUM_SECONDS);
    pa.sleep(NUM_SECONDS * 1_000);

    stream.stop()?;
    stream.close()?;

    println!("Test finished.");

    Ok(())
}
