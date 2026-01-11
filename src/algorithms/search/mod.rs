//! # Search Algorithms Module
//!
//! ## 🎯 Purpose & Motivation
//!
//! This module implements quantum search and amplitude manipulation algorithms
//! that demonstrate quadratic speedup over classical search methods.
//!
//! ## Algorithms Included
//!
//! | Algorithm | Classical | Quantum | Speedup |
//! |-----------|-----------|---------|---------|
//! | Grover's Search | O(N) | O(√N) | Quadratic |
//! | Amplitude Estimation | O(1/ε²) | O(1/ε) | Quadratic |
//! | Quantum Counting | O(N) | O(√N) | Quadratic |
//!
//! ## 📚 References
//!
//! - Grover, L. (1996). "A fast quantum mechanical algorithm for database search"
//! - Brassard et al. (2002). "Quantum Amplitude Amplification and Estimation"

pub mod grovers;
pub mod amplitude_estimation;

pub use grovers::*;
pub use amplitude_estimation::*;
