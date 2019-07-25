use libm::F32Ext;

static SAMPLE_RATE : f32 = 44100.0;

fn lin_interpolate(val_1:f32, val_2:f32, location:f32) -> f32{
    val_1 * (1.0 - location) + val_2 * location
}

pub trait AudioComponent<T: Copy, V> {
    fn initial_state(&self) -> V;
    fn step(&mut self, input: T);
    fn sample(&self, input: T) -> V;
    fn step_and_sample(&mut self, input: T) -> V { self.step(input); self.sample(input) }
}

pub struct NaiveTableOsc {
    cur_index: f32,
    table: &'static [f32],
    table_increment: f32
}

impl NaiveTableOsc {
    pub fn new(table: &'static [f32]) -> NaiveTableOsc {
        NaiveTableOsc {
            cur_index: 0.0,
            table,
            table_increment: table.len() as f32 / SAMPLE_RATE
        }
    }
}

impl AudioComponent<(f32, f32, f32), f32> for NaiveTableOsc {
    fn initial_state(&self) -> f32 { 0.0 }

    fn step(&mut self, (freq, _amp, _add): (f32, f32, f32)) {
        let phase_increment = freq * self.table_increment;
        self.cur_index += phase_increment;
    }

    fn sample(&self, (_freq, amp, add): (f32, f32, f32)) -> f32 {
        let fract_part = self.cur_index - self.cur_index.floor();
        let int_part = self.cur_index as usize;
        let int_part = int_part % self.table.len();
        let next = (int_part + 1) % self.table.len();

        let val = lin_interpolate(self.table[int_part],
            self.table[next],
            fract_part
        );
        val * amp + add
    }
}

pub struct TanHWaveshaper {
}

impl TanHWaveshaper {
    pub fn new() -> Self {
        TanHWaveshaper {}
    }
}

impl AudioComponent<(f32, f32), f32> for TanHWaveshaper {
    fn initial_state(&self) -> f32 { 0.0 }

    fn step(&mut self, (_input, _drive): (f32, f32)) {}

    fn sample(&self, (input, drive): (f32, f32)) -> f32 {
        (input * drive).tanh() / drive.tanh()
    }
}