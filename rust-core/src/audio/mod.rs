//! Audio input/output management with cpal

pub mod input;
pub mod output;
pub mod buffer;
pub mod processor;
pub mod gate;

pub use input::AudioInput;
pub use output::AudioOutput;
pub use buffer::AudioRingBuffer;
pub use processor::AudioProcessor;
