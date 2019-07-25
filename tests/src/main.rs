extern crate portaudio;

use portaudio as pa;
use audio_engine::audio_objects::{
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

    let mut modmod = NaiveTableOsc::new(&0.3, &300.0, &660.0, &SINE_2048);
    let mut modulator = NaiveTableOsc::new(modmod.get_value_chan(), &220.0, &440.0, &SINE_2048);
    let mut sine_osc = NaiveTableOsc::new(modulator.get_value_chan(), &1.0, &0.0, &SINE_2048);
    let mut disto_mod = NaiveTableOsc::new(&2.3, &3.02, &3.2, &TRIANGLE_2);
    let mut last = TanHWaveshaper::new(sine_osc.get_value_chan(), disto_mod.get_value_chan());
    let mut last = NaiveTableOsc::new(&440.0 ,&1.0, &0.0, &SINE_2048);
    let pa = pa::PortAudio::new()?;
    let settings =
        pa.default_output_stream_settings(CHANNELS, SAMPLE_RATE, FRAMES_PER_BUFFER)?;


    let callback = move |pa::OutputStreamCallbackArgs { buffer, frames, .. }| {
        let mut idx = 0;
        for _ in 0..frames {
            let samp = {*last.get_value_chan()};

            buffer[idx] = samp;
            buffer[idx + 1] = samp;
            idx += 2;

            modmod.next();
            modulator.next();
            sine_osc.next();
            disto_mod.next();
            last.next();
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
