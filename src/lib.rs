//! Rustiq: A comprehensive quantum circuit synthesis library
//!
//! This library provides implementations of various quantum computing techniques:
//!
//! ## Modules
//!
//! - [`gates`] - Core quantum gates and decomposition algorithms
//! - [`algorithms`] - Quantum algorithms (QFT, Grover, arithmetic circuits)
//! - [`optimization`] - Circuit optimization (gate cancellation, T-count, CNOT minimization)
//! - [`error_correction`] - Error correcting codes (bit-flip, Shor, Steane, surface codes)
//! - [`variational`] - Variational algorithms (VQE ansätze, QAOA)
//! - [`synthesis`] - Advanced synthesis (amplitude encoding, state preparation)
//! - [`analysis`] - Circuit analysis (depth, resource estimation)
//! - [`error_mitigation`] - Error mitigation techniques (ZNE, PEC, CDR)
//! - [`cutting`] - Circuit cutting and distribution

pub mod interface;

// New quantum computing modules
pub mod gates;
pub mod algorithms;
pub mod optimization;
pub mod error_correction;
pub mod variational;
pub mod synthesis;
pub mod analysis;
pub mod error_mitigation;
pub mod cutting;
