//! Analysis Module
//!
//! ## 🎯 Why is this used?
//! This module provides the tools to inspect and validate quantum circuits. It 
//! allows for rigorous benchmarking of resource usage, which is key for 
//! both algorithm research and hardware deployment planning.
//!
//! ## ⚙️ How it works?
//! - **Centralized Tools**: Re-exports all analysis functions (gate counting, depth, 
//!   interaction graphs) for a unified inspection API.
//!
//! ## 📍 Where to apply this?
//! Use this module at the end of a synthesis or optimization pipeline to 
//! generate reports on the quality of the resulting circuit.
//!
//! ## 📊 Code Behavior
//! - Structural wrapper with zero overhead.

pub mod circuit_analysis;

pub use circuit_analysis::*;
