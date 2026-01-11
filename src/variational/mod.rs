//! Variational Module
//!
//! ## 🎯 Why is this used?
//! This module provides the tools for hybrid quantum-classical algorithms. 
//! These are the most promising algorithms for the current NISQ era as 
//! they can tolerate higher noise levels by offloading part of the 
//! computational burden to a classical optimizer.
//!
//! ## ⚙️ How it works?
//! - **Ansatz Treasury**: Re-exports various parameterized circuit structures 
//!   (HEA, QAOA, UCCSD) and gradient calculation tools.
//!
//! ## 📍 Where to apply this?
//! Apply this when solving optimization, chemistry, or machine learning 
//! problems where you need to iteratively tune quantum parameters to 
//! minimize a cost function.
//!
//! ## 📊 Code Behavior
//! - Structural wrapper with zero overhead.

pub mod ansatz;
pub mod qaoa_variants;
pub mod vqe_variants;

pub use ansatz::*;
pub use qaoa_variants::*;
pub use vqe_variants::*;
