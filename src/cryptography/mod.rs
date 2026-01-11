//! # Post-Quantum Cryptography Module
//!
//! ## 🎯 Purpose & Motivation
//!
//! This module implements **NIST-standardized Post-Quantum Cryptography (PQC)** algorithms
//! designed to resist attacks from both classical and quantum computers. With the
//! advancement of quantum computing (Shor's algorithm threatens RSA/ECC), PQC is
//! essential for future-proof security.
//!
//! ## 🔒 NIST PQC Standards (2024-2025)
//!
//! | Algorithm | Type | NIST Standard | Status |
//! |-----------|------|---------------|--------|
//! | ML-KEM (Kyber) | Key Encapsulation | FIPS 203 | Finalized Aug 2024 |
//! | ML-DSA (Dilithium) | Digital Signature | FIPS 204 | Finalized Aug 2024 |
//! | SLH-DSA (SPHINCS+) | Digital Signature | FIPS 205 | Finalized Aug 2024 |
//! | FN-DSA (FALCON) | Digital Signature | Draft 2024 | Pending |
//! | HQC | Key Encapsulation | Selected Mar 2025 | Backup |
//!
//! ## ⚙️ Cryptographic Foundations
//!
//! ### Lattice-Based Cryptography
//!
//! Most NIST-selected algorithms use **lattice problems**:
//! - **Learning With Errors (LWE)**: Given (A, b = As + e mod q), find s
//! - **Ring-LWE**: LWE over polynomial rings for efficiency
//! - **Module-LWE**: Generalization providing security/efficiency trade-offs
//!
//! These problems are believed to be hard even for quantum computers.
//!
//! ### Why Lattice-Based?
//!
//! 1. **Security**: Based on worst-case hardness of lattice problems
//! 2. **Efficiency**: Competitive key/signature sizes and speeds
//! 3. **Versatility**: Supports encryption, signatures, and advanced crypto
//!
//! ## 📍 Use Cases
//!
//! - **TLS 1.3**: Hybrid key exchange (classical + PQC)
//! - **Code Signing**: Quantum-resistant software authentication
//! - **Long-term Secrets**: Data that must remain secure for decades
//! - **Government/Military**: Compliance with NSA CNSA 2.0 requirements
//!
//! ## 🔬 2025-26 Research Context
//!
//! - **Hybrid Encryption**: Combining ML-KEM with X25519 for transition
//! - **qLDPC Integration**: Future error-corrected quantum implementations
//! - **Side-Channel Resistance**: Constant-time implementations critical
//!
//! ## 📚 References
//!
//! - NIST FIPS 203: ML-KEM (2024)
//! - NIST FIPS 204: ML-DSA (2024)
//! - NIST FIPS 205: SLH-DSA (2024)
//! - CRYSTALS-Kyber: https://pq-crystals.org/kyber/
//! - CRYSTALS-Dilithium: https://pq-crystals.org/dilithium/

pub mod lattice;
pub mod ml_kem;
pub mod ml_dsa;
pub mod lwe;

pub use lattice::*;
pub use ml_kem::*;
pub use ml_dsa::*;
pub use lwe::*;
