use libm::F32Ext;

static SAMPLE_RATE : f32 = 44100.0;

//not sure interpolate should be there, will probably move it later
fn lin_interpolate(val_1:f32, val_2:f32, location:f32) -> f32{
    val_1 * (1.0 - location) + val_2 * location
}

pub trait Audio {
    fn next(&mut self) -> f32;
}

// Adds the Audio traits to be able to pass numerics to audio objects as parameters.
impl Audio for f32 {
    fn next(&mut self) -> f32 {
        *self
    }
}

impl Audio for i32 {
    fn next(&mut self) -> f32 {
        *self as f32
    }
}


pub struct NaiveTableOsc<T:Audio, U:Audio, V:Audio> {
    cur_index: f32,
    pub freq: T,
    pub amp: U,
    pub add: V,
    pub table: &'static [f32],
    table_increment: f32,
}
impl <T:Audio, U:Audio, V:Audio> NaiveTableOsc<T, U, V> {
    pub fn new(freq: T, amp:U, add:V, table: &'static [f32]) -> NaiveTableOsc<T, U, V> {
        NaiveTableOsc{cur_index:0.0,freq:freq, amp:amp, add:add, table:table, table_increment:table.len() as f32 / SAMPLE_RATE}
    }
}

impl <T:Audio, U:Audio, V:Audio> Audio for NaiveTableOsc<T, U, V> {
    fn next(&mut self) -> f32{
        //gets the next samples for freq and mul.
        let amp = self.amp.next();
        let add = self.add.next();

        let fract_part = self.cur_index - self.cur_index.floor();
        let mut int_part = self.cur_index as usize;
        let int_part = int_part % self.table.len();
        let next = (int_part + 1) % self.table.len();

        let val = lin_interpolate(self.table[int_part],
            self.table[next],
            fract_part
        );

        let phase_increment = self.freq.next() * self.table_increment;
        self.cur_index = int_part as f32 + fract_part + phase_increment;
        val * amp + add
    }
}


pub struct TanHWaveshaper<T:Audio, U:Audio> {
    input : T,
    pub drive: U,
}

impl <T:Audio, U:Audio> TanHWaveshaper<T, U> {
    pub fn new(input: T, drive: U) -> Self{
        TanHWaveshaper{input:input, drive:drive}
    }
}

impl <T:Audio, U:Audio> Audio for TanHWaveshaper<T, U> {
    fn next(&mut self) -> f32 {
        let drive = self.drive.next();
        (self.input.next() * drive).tanh()/drive.tanh()
    }
}
