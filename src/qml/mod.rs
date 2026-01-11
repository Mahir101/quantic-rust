//! # Quantum Machine Learning Module
//!
//! ## 🎯 Purpose & Motivation
//!
//! This module implements cutting-edge **Quantum Machine Learning (QML)** algorithms
//! based on 2025-2026 research breakthroughs. QML leverages quantum parallelism
//! and interference to potentially accelerate machine learning tasks.
//!
//! ## 🔬 2025-26 Research Context
//!
//! - **Google Quantum AI (2025)**: First generative quantum advantage
//! - **Quantum Neural Networks**: VQC, QCNN, QRNN architectures
//! - **Variational Circuits**: Barren plateau mitigation strategies
//! - **Quantum Kernels**: QSVM, projected quantum kernels
//!
//! ## Submodules
//!
//! - [`neural_networks`] - Quantum Neural Network architectures (VQC, QCNN)
//! - [`kernels`] - Quantum kernel methods (QSVM)
//! - [`generative`] - Quantum generative models (QBM, QGAN)
//!
//! ## 📚 References
//!
//! - Cerezo et al. (2021). "Variational Quantum Algorithms"
//! - Schuld & Petruccione (2021). "Machine Learning with Quantum Computers"
//! - Google (2025). "Generative Quantum Advantage"

pub mod neural_networks;

pub use neural_networks::*;
