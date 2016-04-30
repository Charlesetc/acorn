// utils.rs

use std::result;

// position: Position,
pub struct Error { pub description: String }

pub type Result = result::Result<(), Error>;
