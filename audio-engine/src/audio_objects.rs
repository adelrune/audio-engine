use libm::F32Ext;

static SAMPLE_RATE : f32 = 44100.0;

pub struct ValueChan<T> {
    value: *const T
}

impl<T> ValueChan<T> {
    pub fn from(from: *const T) -> ValueChan<T> {
        ValueChan {
            value: from
        }
    }

    pub fn get(&self) -> &T {
        unsafe {self.value.as_ref().unwrap()}
    }
}


//not sure interpolate should be there, will probably move it later
fn lin_interpolate(val_1:f32, val_2:f32, location:f32) -> f32{
    val_1 * (1.0 - location) + val_2 * location
}

// Adds the Audio traits to be able to pass numerics to audio objects as parameters.
pub struct ConstantOsc {
    value: f32
}

impl ConstantOsc {
    pub fn get_value_chan(&self) -> ValueChan<f32> {
        ValueChan::from(&self.value)
    }
}

impl From<i32> for ConstantOsc {
    fn from(value: i32) -> ConstantOsc {
        ConstantOsc {
            value: value as f32
        }
    }
}

impl From<f32> for ConstantOsc {
    fn from(value: f32) -> ConstantOsc {
        ConstantOsc {
            value
        }
    }
}

pub struct NaiveTableOsc {
    cur_index: f32,
    pub freq: ValueChan<f32>,
    pub amp: ValueChan<f32>,
    pub add: ValueChan<f32>,
    pub table: &'static [f32],
    table_increment: f32,
    current_value: f32
}

impl NaiveTableOsc {
    pub fn new(freq: ValueChan<f32>, amp: ValueChan<f32>, add: ValueChan<f32>, table: &'static [f32]) -> Self {
        NaiveTableOsc {
            cur_index: 0.0,
            freq, 
            amp, 
            add, 
            table, 
            table_increment: table.len() as f32 / SAMPLE_RATE,
            current_value: 0.0
        }
    }

    pub fn get_value_chan(&self) -> ValueChan<f32> {
        ValueChan::from(&self.current_value)
    }

    pub fn next(&mut self) {
        let freq = self.freq.get();
        let phase_increment = freq * self.table_increment;
        self.cur_index += phase_increment;

            //gets the next samples for freq and mul.
        let amp = self.amp.get();
        let add = self.add.get();

        let fract_part = self.cur_index - self.cur_index.floor();
        let int_part = self.cur_index as usize;
        let int_part = int_part % self.table.len();
        let next = (int_part + 1) % self.table.len();

        let val = lin_interpolate(self.table[int_part],
            self.table[next],
            fract_part
        );
        self.current_value = val * amp + add;
    }
}


pub struct TanHWaveshaper {
    input : ValueChan<f32>,
    pub drive: ValueChan<f32>,
    current_value: f32
}

impl TanHWaveshaper {
    pub fn new(input: ValueChan<f32>, drive: ValueChan<f32>) -> Self {
        TanHWaveshaper {
            input, 
            drive,
            current_value: 0.0
        }
    }

    pub fn get_value_chan(&self) -> ValueChan<f32> {
        ValueChan::from(&self.current_value)
    }

    pub fn next(&mut self) {
        let drive = self.drive.get();
        self.current_value = (self.input.get() * drive).tanh()/drive.tanh();
    }
}