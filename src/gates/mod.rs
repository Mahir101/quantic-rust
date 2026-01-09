//! # Gates Module
//!
//! Gates module - Quantum gate definitions and decomposition algorithms
//!
//! ## 🎯 Why is this used?
//! This is the entry point for all gate-related operations. It groups together the symbolic 
//! definitions of quantum gates and the mathematical tools required to manipulate them.
//!
//! ## ⚙️ How it works?
//! - **Submodule Organization**: Re-exports `core` (data structures) and `decomposition` 
//!   (algorithms) for a flattened, user-friendly API.
//! - **Consistency**: Ensures that all gate manipulations share a unified coordinate system 
//!   and precision settings.
//!
//! ## 📍 Where to apply this?
//! Import this module whenever you need to define a circuit or perform low-level 
//! unitary transformations.
//!
//! ## 📊 Code Behavior
//! - Purely structural module. No runtime overhead other than re-exports.

pub mod core;
pub mod decomposition;

pub use core::*;
pub use decomposition::*;
