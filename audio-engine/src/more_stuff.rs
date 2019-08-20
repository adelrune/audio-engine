use crate::utils;
use hound;
use std::i16;


pub struct FileReader {
    samples: Vec<f32>,
    cur_index: f32,
}

impl FileReader {
    // right now this is left channel mono only
    // this assumes that the sampling rate is the same as declared in utils and does not correct the rate
    // this also assumes that we are dealing with 16bit samples
    pub fn new(file_path: &str) -> Self {
        let mut reader = hound::WavReader::open(file_path).unwrap();
        let num_chans = reader.spec().channels;
        let samples = reader.samples::<i16>().enumerate().filter(|(idx, _)| idx % num_chans as usize == 0).map(|(_, sample)| {
            let max_amplitude = i16::MAX as f32;
            (sample.unwrap() as f32) / max_amplitude
        }).collect();

        FileReader {
            cur_index:0.0,
            samples:samples
        }
    }

    pub fn next(&mut self, rate:f32, amp:f32) -> f32 {
        // this is mostly copy pasta from the naivetableosc code. Find a way to reuse later
        let fract_part = self.cur_index - self.cur_index.floor();
        let mut int_part = self.cur_index as usize;
        int_part = int_part % self.samples.len() as usize;
        let next = (int_part + 1) % self.samples.len();

        let val = utils::lin_interpolate(self.samples[int_part],
            self.samples[next],
            fract_part
        );

        let phase_increment = rate;
        self.cur_index += phase_increment;
        val * amp

    }
}
