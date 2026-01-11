//! Quantic-Rust: High-Performance Quantum Computing Library
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
//!
//! ## 🎯 Why is this used?
//! Quantic-Rust is designed to be a "瑞士军刀" (Swiss Army Knife) for quantum 
//! software engineering. It combines high-performance Rust execution with a 
//! rich set of features covering the entire quantum stack—from gate-level 
//! primitives to fault-tolerant code design and distributed circuit cutting.
//!
//! ## ⚙️ How it works?
//! - **Hierarchical Composition**: Low-level `gates` form the basis for `algorithms`, 
//!    which are then `optimized` and further protected by `error_correction` or `mitigation`.
//! - **Unified Representation**: Uses a consistent internal format for circuits, 
//!   enabling seamless interaction between synthesis, analysis, and optimization tools.
//! - **Extensibility**: Each module is built to be independent yet interoperable, 
//!   allowing for easy research experimentation or production-grade integration.
//!
//! ## 📍 Where to apply this?
//! - **Algorithm Research**: Developing and testing new circuit synthesis/optimization rules.
//! - **Hardware Backend Development**: Using the transpilation and cutting features 
//!   to map circuits to physical hardware topologies.
//! - **Quantum App Development**: Building high-level applications (VQE, HHL) 
//!   with production-ready performance.
//!
//! ## 📊 Code Behavior
//! - **Efficiency**: Leverages Rust's memory safety and zero-cost abstractions for 
//!   fast circuit processing even for large qubit counts.
//! - **Thread-Safety**: Core data structures are designed for parallel analysis 
//!   and optimization (Rayon-friendly).

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

// Post-Quantum Cryptography (NIST FIPS 203/204/205)
pub mod cryptography;

// Quantum Machine Learning (2025-26 algorithms)
pub mod qml;
