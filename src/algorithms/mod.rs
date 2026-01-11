//! Algorithms Module - Quantum algorithms and circuit constructions
//!
//! ## 🎯 Why is this used?
//! This module acts as the central registry for high-level quantum procedures. It organizes 
//! complex routines into logical categories (Arithmetic, Linear Systems, etc.), allowing 
//! users to easily compose large-scale quantum applications from verified building blocks.
//!
//! ## ⚙️ How it works?
//! - **Modularity**: Explodes the library into specific sub-domains of quantum logic.
//! - **Public API**: Re-exports all submodules for a flattened access pattern.
//!
//! ## 📍 Where to apply this?
//! Import this module for implementing algorithms that go beyond simple gate manipulations, 
//! such as phase estimation or Hamiltonian evolution.
//!
//! ## 📊 Code Behavior
//! - Primary structural role with zero runtime overhead.

pub mod qft;
pub mod arithmetic;
pub mod quantum_walk;
pub mod linear_systems;
pub mod simulations;
pub mod oracles;
pub mod search;
pub mod phase_estimation;
pub mod qsvt;
pub mod novel_2025;

pub use qft::*;
pub use arithmetic::*;
pub use quantum_walk::*;
pub use linear_systems::*;
pub use simulations::*;
pub use oracles::*;
pub use search::*;
pub use phase_estimation::*;
pub use qsvt::*;
pub use novel_2025::*;
