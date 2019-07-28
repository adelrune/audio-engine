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

struct SignalChain {
    modmod: NaiveTableOsc,
    modulator: NaiveTableOsc,
    sine_osc: NaiveTableOsc,
    disto_mod: NaiveTableOsc,
    output: TanHWaveshaper,
    output_states: [f32;5]
}

impl SignalChain {
    pub fn new() -> Self {
        SignalChain {
            modmod:NaiveTableOsc::new(&SINE_2048),
            modulator:NaiveTableOsc::new(&SINE_2048),
            sine_osc:NaiveTableOsc::new(&SINE_2048),
            disto_mod:NaiveTableOsc::new(&TRIANGLE_2),
            output:TanHWaveshaper::new(),
            output_states: [0.0;5],
        }
    }

    fn next(&mut self) -> f32 {
        self.output_states[0] = self.modmod.next(0.3, 300.0, 660.0);
        self.output_states[1] = self.modulator.next(self.output_states[0], 220.0, 440.0);
        self.output_states[2] = self.sine_osc.next(self.output_states[1], 1.0, 0.0);
        self.output_states[3] = self.disto_mod.next(2.3, 3.0, 3.2);
        self.output_states[4] = self.output.next(self.output_states[2] + 0.2 * self.output_states[4], self.output_states[3]);
        self.output_states[4]
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
