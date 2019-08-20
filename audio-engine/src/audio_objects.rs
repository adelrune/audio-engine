use libm::F32Ext;
use crate::utils;

pub struct NaiveTableOsc {
    cur_index: f32,
    table: &'static [f32],
    table_increment: f32
}

impl NaiveTableOsc {
    pub fn new(table: &'static [f32]) -> Self {
        NaiveTableOsc {
            cur_index: 0.0,
            table,
            table_increment: table.len() as f32 / utils::SAMPLE_RATE
        }
    }
    pub fn next(&mut self, freq:f32, amp:f32, add:f32) -> f32 {
        let fract_part = self.cur_index - self.cur_index.floor();
        let mut int_part = self.cur_index as usize;
        int_part = int_part % self.table.len();
        let next = (int_part + 1) % self.table.len();

        let val = utils::lin_interpolate(self.table[int_part],
            self.table[next],
            fract_part
        );


        let phase_increment = freq * self.table_increment;
        self.cur_index += phase_increment;

        val * amp + add
    }
}

pub struct TanHWaveshaper {}

impl TanHWaveshaper {
    pub fn new() -> Self {
        TanHWaveshaper {}
    }
    pub fn next(&mut self, input:f32, drive:f32) -> f32 {
        (input * drive).tanh() / drive.tanh()
    }
}
