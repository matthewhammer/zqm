// Serde: Persistent state between invocations of ZQM
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub struct Grid {}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub struct Editor {}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub enum Command {}
