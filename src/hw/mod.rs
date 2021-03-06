/// Provides emulation for the [MOS Technology 6502](http://en.wikipedia.org/wiki/MOS_Technology_6502) processor
///
/// This processor is commonly found in the [Nintendo Entertainment System](http://en.wikipedia.org/wiki/Nintendo_Entertainment_System)
/// (in the form of the slightly modified Ricoh 2A03), the [Apple II](http://en.wikipedia.org/wiki/Apple_II), 
/// [8-bit Atari units](http://en.wikipedia.org/wiki/Atari_8-bit_family), and the [Commodore 64](http://en.wikipedia.org/wiki/Commodore_64)
pub mod mos6502;

/// Provides emulation for the [Ricoh RP2C02 Picture Processing Unit (PPU)](https://en.wikipedia.org/wiki/Picture_Processing_Unit) as well
/// as the RP2C07 (for PAL television systems)
///
/// This processor is found in the [Nintendo Entertainment System](http://en.wikipedia.org/wiki/Nintendo_Entertainment_System)
#[allow(non_snake_case)]
pub mod rp2C02;
