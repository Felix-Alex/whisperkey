pub mod encoder;
pub mod recorder;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopReason {
    Manual,
    Silence,
    MaxDuration,
}
