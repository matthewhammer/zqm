// Serde: Persistent state between invocations of ZQM
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Chain { }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Editor { }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Command { }
