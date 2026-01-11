//! # Oracle-Based Quantum Algorithms Module
//!
//! ## 🎯 Purpose & Motivation
//!
//! This module implements the foundational oracle-based quantum algorithms that
//! demonstrate quantum computational advantage. These algorithms represent the
//! theoretical bedrock of quantum computing, proving that quantum computers can
//! solve certain problems exponentially faster than any classical algorithm.
//!
//! ## ⚙️ Algorithmic Overview
//!
//! Oracle-based algorithms use a "black-box" function (oracle) that can be queried
//! in quantum superposition. This allows quantum algorithms to extract global
//! properties of the function with fewer queries than classically possible.
//!
//! ### Algorithms Included:
//!
//! | Algorithm | Classical Queries | Quantum Queries | Speedup |
//! |-----------|-------------------|-----------------|---------|
//! | Deutsch's | 2 | 1 | 2x |
//! | Deutsch-Jozsa | 2^(n-1) + 1 | 1 | Exponential |
//! | Bernstein-Vazirani | n | 1 | Linear |
//! | Simon's | O(2^n) | O(n) | Exponential |
//!
//! ## 📍 Use Cases
//!
//! - **Educational**: Understanding quantum parallelism and interference
//! - **Benchmarking**: Testing quantum hardware capabilities
//! - **Research**: Foundation for more complex algorithms (Shor's, Grover's)
//!
//! ## 📚 References
//!
//! - Deutsch, D. (1985). "Quantum theory, the Church-Turing principle and the universal quantum computer"
//! - Deutsch, D. & Jozsa, R. (1992). "Rapid solution of problems by quantum computation"
//! - Bernstein, E. & Vazirani, U. (1993). "Quantum complexity theory"
//! - Simon, D. (1994). "On the power of quantum computation"

pub mod deutschs;
pub mod deutsch_jozsa;
pub mod bernstein_vazirani;
pub mod simons;

pub use deutschs::*;
pub use deutsch_jozsa::*;
pub use bernstein_vazirani::*;
pub use simons::*;
