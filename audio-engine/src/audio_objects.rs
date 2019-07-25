use libm::F32Ext;

static SAMPLE_RATE : f32 = 44100.0;


//not sure interpolate should be there, will probably move it later
fn lin_interpolate(val_1:f32, val_2:f32, location:f32) -> f32{
    val_1 * (1.0 - location) + val_2 * location
}

// Adds the Audio traits to be able to pass numerics to audio objects as parameters.

pub struct NaiveTableOsc {
    cur_index: f32,
    pub freq: *const f32,
    pub amp: *const f32,
    pub add: *const f32,
    pub table: &'static [f32],
    table_increment: f32,
    current_value: f32
}

pub fn get_val(val : *const f32) -> f32 {
    *unsafe {val.as_ref().unwrap()}
}

impl NaiveTableOsc {
    pub fn new(freq: &f32, amp: &f32, add: &f32, table: &'static [f32]) -> Self {
        NaiveTableOsc {
            cur_index: 0.0,
            freq: freq as *const f32,
            amp: amp as *const f32,
            add: add as *const f32,
            table:table,
            table_increment: table.len() as f32 / SAMPLE_RATE,
            current_value: 0.0
        }
    }

    pub fn get_value_chan(&self) -> &f32 {
        &self.current_value
    }

    pub fn next(&mut self) {
        let freq = get_val(self.freq);

            //gets the next samples for freq and mul.
        let amp = get_val(self.amp);
        let add = get_val(self.add);

        let fract_part = self.cur_index - self.cur_index.floor();
        let int_part = self.cur_index as usize;
        let int_part = int_part % self.table.len();
        let next = (int_part + 1) % self.table.len();

        let val = lin_interpolate(self.table[int_part],
            self.table[next],
            fract_part
        );

        let phase_increment = freq * self.table_increment;
        self.cur_index += phase_increment;

        self.current_value = val * amp + add;
    }
}


pub struct TanHWaveshaper {
    input : *const f32,
    pub drive: *const f32,
    current_value: f32
}

impl TanHWaveshaper {
    pub fn new(input: &f32, drive: &f32) -> Self {
        TanHWaveshaper {
            input: input as *const f32,
            drive: drive as *const f32,
            current_value: 0.0
        }
    }

    pub fn get_value_chan(&self) -> &f32 {
        &self.current_value
    }

    pub fn next(&mut self) {
        let drive = get_val(self.drive);
        self.current_value = (get_val(self.input) * drive).tanh()/drive.tanh();
    }
}
