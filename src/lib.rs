mod bridge;
mod native;

// TODO: Figure out an error handling strategy once we have more errors.
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

// TODO: Just export all the native stuff for now.
pub use native::*;

