// Компоненты бывших публичных crate сохраняют полный API как внутреннюю границу;
// часть методов будет подключена последующими продуктовыми задачами.
#![allow(dead_code)]

pub mod command;
pub mod core;
pub mod host;
pub mod kafka;
pub mod lifecycle;
pub mod policy;
pub mod runtime;
pub mod telemetry;
