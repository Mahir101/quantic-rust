//! Error Correction Module
//!
//! ## 🎯 Why is this used?
//! This is the high-reliability hub of the library. It provides the necessary 
//! tools to escape the NISQ era and move toward logical, fault-tolerant 
//! computation. It handles everything from encoded data representation to 
//! error inference via decoders.
//!
//! ## ⚙️ How it works?
//! - **Categorization**: Groups codes (Surface, Shor), operations (Lattice Surgery), 
//!   and inference engines (Decoders) into a single, cohesive namespace.
//!
//! ## 📍 Where to apply this?
//! Integrate this into any long-running quantum computation where individual 
//! gate errors will likely accumulate and destroy the calculation's validity.
//!
//! ## 📊 Code Behavior
//! - Structural organization module with zero runtime overhead.

pub mod codes;
pub mod lattice_surgery;
pub mod ldpc;
pub mod decoders;
pub mod advanced_codes;

pub use codes::*;
pub use lattice_surgery::*;
pub use ldpc::*;
pub use decoders::*;
pub use advanced_codes::*;
