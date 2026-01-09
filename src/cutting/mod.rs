//! Cutting Module
//!
//! ## 🎯 Why is this used?
//! This module provides the infrastructure for distributed quantum computing and 
//! circuit partitioning. It enables the execution of circuits that are physically 
//! too large for a single QPU.
//!
//! ## ⚙️ How it works?
//! - **Partitioning Logic**: Re-exports cutting algorithms that decompose wires 
//!   and gates into sets of locally-executable operations.
//!
//! ## 📍 Where to apply this?
//! Apply this at the post-optimization stage when the logical circuit must be 
//! mapped to a cluster of smaller quantum devices.
//!
//! ## 📊 Code Behavior
//! - Structural wrapper with zero overhead.

pub mod cutting;

pub use cutting::*;
