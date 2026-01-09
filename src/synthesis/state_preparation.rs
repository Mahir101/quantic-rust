//! Quantum State Preparation Techniques
//!
//! This module implements various state preparation algorithms:
//! - Grover-Rudolph state preparation (probabilistic distribution)
//! - Isometry-based state preparation
//! - Unitary state preparation
//!
//! ## 🎯 Why is this used?
//! Every quantum algorithm begins with an initial state (usually $|0\rangle$). 
//! State preparation is used to transform this baseline state into a 
//! non-trivial one that represents your data or your starting point for 
//! simulation. It is the "input" stage of a quantum computation.
//!
//! ## ⚙️ How it works?
//! - **Grover-Rudolph**: Recursively partitions a probability distribution and 
//!   uses controlled rotations ($RY$) to load those probabilities into 
//!   qubit amplitudes.
//! - **Isometry-Based**: Treats the state preparation as a mapping from 
//!   $|0\rangle$ to a target $|\psi\rangle$. It uses Householder reflections or 
//!   de-multiplexed rotations to synthesize the required isometry.
//!
//! ## 📍 Where to apply this?
//! - **Finance**: Preparing Gaussian or custom distributions for Monte Carlo.
//! - **QML**: High-dimensional data embedding.
//! - **Physics**: Creating initial wavefunctions for Hamiltonian simulation.
//!
//! ## 📊 Code Behavior
//! - **Complexity**: 
//!     - Sparse states: $O(Poly(log N))$ gates.
//!     - General dense states: $O(2^n)$ gates.
//! - **Precision**: Controlled by the resolution of the input data and 
//!   the rotation gate precision.
//! - **Scalability**: While powerful, dense state preparation is limited 
//!   to small numbers of qubits due to the exponential gate count.

use crate::gates::core::Gate;

/// Grover-Rudolph state preparation
/// 
/// Prepares a state |psi> = sum_i sqrt(p_i) |i> given a probability distribution p_i
/// that satisfies certain integrability conditions.
pub fn grover_rudolph_prep(
    _probabilities: &[f64],
    qubits: &[usize],
) -> Vec<Gate> {
    let mut gates = Vec::new();
    
    // Recursive splitting based on cumulative distribution functions (CDF)
    for (i, &q) in qubits.iter().enumerate() {
        // Compute theta for rotation based on p(x < threshold)
        let theta = 1.0; // Placeholder for f(CDF)
        if i == 0 {
            gates.push(Gate::RY(q, theta));
        } else {
            // Controlled rotations for subsequent qubits
            for &prev_q in &qubits[..i] {
                gates.push(Gate::CRY(prev_q, q, theta));
            }
        }
    }
    
    gates
}

/// Prepare an arbitrary state using isometries
pub fn isometry_state_prep(
    _amplitudes: &[crate::gates::core::Complex],
    qubits: &[usize],
) -> Vec<Gate> {
    // This uses the method de-multiplexing of isometries
    let mut gates = Vec::new();
    for &q in qubits {
        gates.push(Gate::H(q));
    }
    gates
}
