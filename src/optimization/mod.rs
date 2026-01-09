//! Optimization Module
//!
//! ## 🎯 Why is this used?
//! In the era of noisy quantum hardware, every gate saved is an increase in 
//! result fidelity. This module provides high-performance routines to reduce 
//! gate counts, CNOT depth, and T-depth by finding logically identical but 
//! hardware-friendlier representations of quantum circuits.
//!
//! ## ⚙️ How it works?
//! - **Pipeline approach**: Combines rule-based peephole optimization with 
//!   topological rewrite rules (ZX-calculus) to achieve deep circuit reduction.
//! - **Submodules**: `optimize` handles standard gate-set transformations, while 
//!   `zx_calculus` provides advanced graph-theoretic optimization.
//!
//! ## 📍 Where to apply this?
//! Mandatory step before running any high-depth circuit on physical QPUs. 
//! It bridges the results from `synthesis` to the physical execution layer.
//!
//! ## 📊 Code Behavior
//! - Structural wrapper with zero overhead.

pub mod optimize;
pub mod zx_calculus;

pub use optimize::*;
pub use zx_calculus::*;
