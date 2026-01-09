//! Synthesis Module
//!
//! ## 🎯 Why is this used?
//! This module is the "compiler" of the library. It translates abstract mathematical 
//! objects (unitaries, probability vectors, data arrays) into concrete series of 
//! quantum gates. It is essential for initializing quantum registers and 
//! implementing non-trivial operators.
//!
//! ## ⚙️ How it works?
//! - **Specialized Layout**: Organizes synthesis methods by their target: 
//!   classical data (`qram`), general states (`state_preparation`), and 
//!   complex unitaries (`advanced`).
//!
//! ## 📍 Where to apply this?
//! Use this at the start of your quantum program to load data or define the 
//! system's initial evolution operators.
//!
//! ## 📊 Code Behavior
//! - Structural organization module with zero runtime overhead.

pub mod advanced;
pub mod qram;
pub mod state_preparation;

pub use advanced::*;
pub use qram::*;
pub use state_preparation::*;
