#[macro_use] extern crate signal_macro;


signal_chain!{
    MySignalChain (

        modulator: Naivetableosc(&TRIANGLE_2),
        generator: Naivetableosc(&SINE_2048),
        modifier: TanhWaveshaper(),

    )
    {
        modulator(1.2, 220.0) + lel + 660;
        generator(modulator, whatevershit(3, 4, 5) + 3);
        modifier(generator);
    }
}


pub fn main() {

}
