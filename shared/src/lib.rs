use std::error::Error as StdError;

pub mod config;
pub mod contracts;
pub mod token;
pub mod util;

pub type Error = Box<dyn StdError + Send + Sync>;
