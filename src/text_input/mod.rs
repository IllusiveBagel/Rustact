mod handle;
mod registry;
mod state;
#[cfg(test)]
mod tests;

pub use handle::TextInputHandle;
pub use registry::TextInputs;
pub use state::{TextInputSnapshot, TextInputState};
