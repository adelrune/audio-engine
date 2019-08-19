extern crate portaudio;
#[macro_use] extern crate signal_macro;
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

signal_chain!{
    SignalChain (
        modmod: NaiveTableOsc(&SINE_2048),
        modulator: NaiveTableOsc(&SINE_2048),
        sine_osc: NaiveTableOsc(&SINE_2048),
        disto_mod:NaiveTableOsc(&TRIANGLE_2),
        output: TanHWaveshaper()
    )
    {
        modmod(0.3, 300, 660);
        modulator(modmod, 220, 440);
        sine_osc(modulator, 1.0, 0);
        disto_mod(2.3, 3, 3.2);
        output(sine_osc + 0.2 * output, disto_mod);
    }
}

fn run() -> Result<(), pa::Error> {
    println!(
        "PortAudio Test: output sine wave. SR = {}, BufSize = {}",
        SAMPLE_RATE, FRAMES_PER_BUFFER
    );
    let mut signal_chain = SignalChain::new();

    let pa = pa::PortAudio::new()?;
    let settings =
        pa.default_output_stream_settings(CHANNELS, SAMPLE_RATE, FRAMES_PER_BUFFER)?;


    let callback = move |pa::OutputStreamCallbackArgs { buffer, frames, .. }| {
        let mut idx = 0;
        for _ in 0..frames {
            let samp = signal_chain.next();
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
