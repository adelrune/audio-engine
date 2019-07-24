use libm::F32Ext;

static SAMPLE_RATE : f32 = 44100.0;

//not sure interpolate should be there, will probably move it later
fn lin_interpolate(val_1:f32, val_2:f32, location:f32) -> f32{
    val_1 * (1.0 - location) + val_2 * location
}

pub trait Audio {
    fn next(&mut self) -> ();
    fn get_sample(&self) -> f32;
}

// Adds the Audio traits to be able to pass numerics to audio objects as parameters.
impl Audio for f32 {
    fn next(&mut self) -> () {}
    fn get_sample(&self) -> f32{
        *self
    }
}

impl Audio for i32 {
    fn next(&mut self) -> () {}
    fn get_sample(&self) -> f32{
        *self as f32
    }
}


pub struct NaiveTableOsc<'a, T:Audio, U:Audio, V:Audio> {
    cur_index: f32,
    pub freq: &'a T,
    pub amp:&'a U,
    pub add:&'a V,
    pub table: &'static [f32],
    table_increment: f32,
}
impl <'a, T:Audio, U:Audio, V:Audio> NaiveTableOsc<'a, T, U, V> {
    pub fn new(freq:&'a T, amp:&'a U, add:&'a V, table: &'static [f32]) -> Self {
        NaiveTableOsc{cur_index:0.0,freq:freq, amp:amp, add:add, table:table, table_increment:table.len() as f32 / SAMPLE_RATE}
    }
}

impl <'a, T:Audio, U:Audio, V:Audio> Audio for NaiveTableOsc<'a, T, U, V> {
    fn get_sample(&self) -> f32 {
        //gets the next samples for freq and mul.
        let amp = self.amp.get_sample();
        let add = self.add.get_sample();

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
    fn next(&mut self) -> () {
        let phase_increment = self.freq.get_sample() * self.table_increment;
        self.cur_index += phase_increment;
    }
}


pub struct TanHWaveshaper<'a, T:Audio, U:Audio> {
    input : &'a T,
    pub drive: &'a U,
}

impl <'a, T:Audio, U:Audio> TanHWaveshaper<'a, T, U> {
    pub fn new(input:&'a T, drive:&'a U) -> Self {
        TanHWaveshaper{input:input, drive:drive}
    }
}

impl <'a, T:Audio, U:Audio> Audio for TanHWaveshaper<'a, T, U> {
    fn get_sample(&self) -> f32 {
        let drive = self.drive.get_sample();
        (self.input.get_sample() * drive).tanh()/drive.tanh()
    }
    fn next(&mut self) -> () {}
}
