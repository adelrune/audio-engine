extern crate portaudio;

use portaudio as pa;
use audio_engine::audio_objects::{
    NaiveTableOsc,
    ConstantOsc,
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
    let stable_0_3 = ConstantOsc::from(0.3);
    let stable_300 = ConstantOsc::from(300);
    let stable_660 = ConstantOsc::from(660);
    let stable_220 = ConstantOsc::from(220);
    let stable_440 = ConstantOsc::from(440);
    let stable_1 = ConstantOsc::from(1);
    let stable_0 = ConstantOsc::from(0);
    let stable_2_3 = ConstantOsc::from(2.3);
    let stable_3 = ConstantOsc::from(3);
    let stable_3_2 = ConstantOsc::from(3.2);


    let mut modmod = NaiveTableOsc::new(stable_0_3.get_value_chan(), stable_300.get_value_chan(), stable_660.get_value_chan(), &SINE_2048);
    let mut modulator = NaiveTableOsc::new(modmod.get_value_chan(), stable_220.get_value_chan(), stable_440.get_value_chan(), &SINE_2048);
    let mut sine_osc = NaiveTableOsc::new(modulator.get_value_chan(), stable_1.get_value_chan(), stable_0.get_value_chan(), &SINE_2048);
    let mut disto_mod = NaiveTableOsc::new(stable_2_3.get_value_chan(), stable_3.get_value_chan(), stable_3_2.get_value_chan(), &TRIANGLE_2);
    let mut last = TanHWaveshaper::new(sine_osc.get_value_chan(), disto_mod.get_value_chan());
    // let mut last = audio_objects::NaiveTableOsc::new(440,1, 0, &TRIANGLE_2);
    let pa = pa::PortAudio::new()?;
    let settings =
        pa.default_output_stream_settings(CHANNELS, SAMPLE_RATE, FRAMES_PER_BUFFER)?;


    let callback = move |pa::OutputStreamCallbackArgs { buffer, frames, .. }| {
        let last_chan = last.get_value_chan();
        let mut idx = 0;
        for _ in 0..frames {
            let samp = *last_chan.get();
    
            modmod.next();
            modulator.next();
            sine_osc.next();
            disto_mod.next();
            last.next();

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
