//! A collection of the data types passed to python.
//! All structures in this crate are bound to Python using [`pyo3`].

pub use tf_demo_parser::demo::data::DemoTick;
pub use tf_demo_parser::demo::message::packetentities::EntityId;
pub use tf_demo_parser::demo::parser::analyser::UserId;

pub mod math;
pub mod game;
pub mod demo;
pub mod entities;
pub mod events;
pub mod analysis;
