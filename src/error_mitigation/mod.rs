//! Error Mitigation Module
//!
//! ## 🎯 Why is this used?
//! This module provides a set of techniques designed to combat the noise in 
//! present-day (NISQ) quantum computers without the heavy qubit overhead 
//! of full error correction.
//!
//! ## ⚙️ How it works?
//! - **Technique Registry**: Groups various extrapolation and regression methods 
//!   that modify circuits and process measurement outcomes to estimate noise-free 
//!   observables.
//!
//! ## 📍 Where to apply this?
//! Whenever you are running algorithms (like VQE or QAOA) on physical quantum 
//! processors and require high-precision results that the underlying hardware 
//! cannot natively provide.
//!
//! ## 📊 Code Behavior
//! - Structural wrapper with zero overhead.

pub mod mitigation;

pub use mitigation::*;
