//! # Quantum Fourier Transform (QFT) Module
//!
//! This module provides:
//! - Standard QFT circuit synthesis
//! - Inverse QFT
//! - Approximate QFT (reduced depth)
//! - Quantum Phase Estimation (QPE)
//!
//! ## 🎯 Why is this used?
//! The QFT is one of the most critical subroutines in quantum computing. It is the quantum 
//! analogue of the discrete Fourier transform and serves as the backbone for many iconic 
//! algorithms, including Shor's factoring algorithm, Phase Estimation, and many quantum 
//! simulation routines.
//!
//! ## ⚙️ How it works?
//! - **Recursive Synthesis**: Constructs the QFT circuit using a sequence of Hadamard 
//!   gates and controlled phase rotations ($R_k$).
//! - **Inverse Mapping**: Implements the Inverse QFT by reversing the gate order and 
//!   negating all rotation phases.
//! - **Depth Optimization**: Provides an **Approximate QFT (AQFT)** version that ignores 
//!   small-angle rotations beyond a certain threshold to reduce circuit depth while 
//!   maintaining high fidelity.
//! - **Phase Estimation**: Implements the QPE wrapper which uses the QFT to extract 
//!   the eigenvalues of a unitary operator.
//!
//! ## 📍 Where to apply this?
//! - **Shor's Algorithm**: For period finding.
//! - **Quantum Chemistry**: In Phase Estimation to find grounds state energies.
//! - **Signal Processing**: For quantum-enhanced frequency analysis.
//! - **State Preparation**: Transforming states from the computational to the Fourier basis.
//!
//! ## 📊 Code Behavior
//! - **Complexity**: 
//!     - Standard QFT: $O(n^2)$ gates for $n$ qubits.
//!     - Approximate QFT: $O(n \log n)$ gates, significantly improving scalability.
//! - **Interaction**: Standard implementation includes a final sequence of SWAP gates 
//!   to correct qubit ordering unless specified otherwise.
//! - **Precision**: Controlled rotations use high-precision $2\pi/2^k$ calculations 
//!   to minimize phase accumulation errors.

use std::f64::consts::PI;
use crate::gates::core::Gate;

// ============================================================================
// QUANTUM FOURIER TRANSFORM (QFT)
// ============================================================================

/// Generate a circuit for the Quantum Fourier Transform on n qubits
/// 
/// QFT transforms computational basis states to Fourier basis:
/// |j⟩ → (1/√N) Σₖ e^(2πijk/N) |k⟩
/// 
/// The circuit consists of Hadamard gates and controlled phase rotations.
/// 
/// # Arguments
/// * `n` - Number of qubits
/// * `qubits` - Optional list of qubit indices (defaults to 0..n)
/// 
/// # Returns
/// A vector of gates implementing the QFT
pub fn qft(n: usize, qubits: Option<&[usize]>) -> Vec<Gate> {
    let q: Vec<usize> = qubits.map(|qs| qs.to_vec())
        .unwrap_or_else(|| (0..n).collect());
    
    assert_eq!(q.len(), n, "Qubit list length must match n");
    
    let mut gates = Vec::new();
    
    for i in 0..n {
        // Apply Hadamard to qubit i
        gates.push(Gate::H(q[i]));
        
        // Apply controlled phase rotations
        for j in (i + 1)..n {
            let k = j - i;
            let angle = PI / (1 << k) as f64;
            gates.push(Gate::CP(q[j], q[i], angle));
        }
    }
    
    // Swap qubits to reverse order (for standard QFT convention)
    for i in 0..(n / 2) {
        gates.push(Gate::SWAP(q[i], q[n - 1 - i]));
    }
    
    gates
}

/// Generate the inverse QFT circuit
/// 
/// The inverse QFT transforms Fourier basis back to computational basis.
/// It's the adjoint of the QFT circuit.
pub fn inverse_qft(n: usize, qubits: Option<&[usize]>) -> Vec<Gate> {
    let q: Vec<usize> = qubits.map(|qs| qs.to_vec())
        .unwrap_or_else(|| (0..n).collect());
    
    assert_eq!(q.len(), n, "Qubit list length must match n");
    
    let mut gates = Vec::new();
    
    // Reverse the swaps first
    for i in 0..(n / 2) {
        gates.push(Gate::SWAP(q[i], q[n - 1 - i]));
    }
    
    // Apply inverse rotations in reverse order
    for i in (0..n).rev() {
        // Apply inverse controlled phase rotations
        for j in ((i + 1)..n).rev() {
            let k = j - i;
            let angle = -PI / (1 << k) as f64;
            gates.push(Gate::CP(q[j], q[i], angle));
        }
        
        // Apply Hadamard (self-inverse)
        gates.push(Gate::H(q[i]));
    }
    
    gates
}

/// Generate an approximate QFT circuit with reduced depth
/// 
/// Approximate QFT ignores small-angle rotations (angle < 2π/2^k for k > cutoff)
/// This reduces circuit depth while maintaining accuracy for many applications.
/// 
/// # Arguments
/// * `n` - Number of qubits
/// * `approximation_degree` - Maximum k for CR_k gates (smaller = more approximate)
/// * `qubits` - Optional qubit indices
pub fn approximate_qft(n: usize, approximation_degree: usize, qubits: Option<&[usize]>) -> Vec<Gate> {
    let q: Vec<usize> = qubits.map(|qs| qs.to_vec())
        .unwrap_or_else(|| (0..n).collect());
    
    let mut gates = Vec::new();
    
    for i in 0..n {
        gates.push(Gate::H(q[i]));
        
        // Only apply rotations up to the approximation degree
        for j in (i + 1)..n {
            let k = j - i;
            if k <= approximation_degree {
                let angle = PI / (1 << k) as f64;
                gates.push(Gate::CP(q[j], q[i], angle));
            }
        }
    }
    
    // Swap qubits
    for i in 0..(n / 2) {
        gates.push(Gate::SWAP(q[i], q[n - 1 - i]));
    }
    
    gates
}

// ============================================================================
// QUANTUM PHASE ESTIMATION (QPE)
// ============================================================================

/// Generate a circuit for Quantum Phase Estimation
/// 
/// QPE estimates the phase φ in the eigenvalue equation U|ψ⟩ = e^(2πiφ)|ψ⟩
/// 
/// # Arguments
/// * `precision_qubits` - Number of qubits for precision (counting register)
/// * `unitary_qubits` - Qubits on which the unitary acts
/// * `controlled_unitary` - A function that generates controlled-U^(2^k) gates
/// 
/// # Returns
/// Gates implementing QPE (requires eigenvector initialization separately)
pub fn qpe_circuit(
    precision_qubits: &[usize],
    _unitary_qubits: &[usize],
    controlled_powers: &[Vec<Gate>],  // controlled_powers[k] = controlled U^(2^k)
) -> Vec<Gate> {
    let n = precision_qubits.len();
    
    assert_eq!(controlled_powers.len(), n, 
        "Must provide controlled U^(2^k) for each precision qubit");
    
    let mut gates = Vec::new();
    
    // Step 1: Apply Hadamard to all precision qubits
    for &q in precision_qubits {
        gates.push(Gate::H(q));
    }
    
    // Step 2: Apply controlled unitary powers
    // Control qubit k applies U^(2^k) to the unitary register
    for (_k, cu_gates) in controlled_powers.iter().enumerate() {
        gates.extend(cu_gates.clone());
    }
    
    // Step 3: Apply inverse QFT to precision register
    gates.extend(inverse_qft(n, Some(precision_qubits)));
    
    gates
}

/// Helper function to create controlled-U circuits for common unitaries
/// This creates c-U^(2^k) by repeated application or efficient synthesis
pub fn create_controlled_power(
    _control: usize,
    base_controlled_u: &[Gate],
    power: usize,
) -> Vec<Gate> {
    // For general U, apply U^(2^k) as repeated squaring
    // This is a simplified version - real implementations use more efficient methods
    let mut gates = Vec::new();
    
    for _ in 0..power {
        gates.extend(base_controlled_u.iter().cloned());
    }
    
    gates
}

// ============================================================================
// GROVER'S SEARCH ALGORITHM
// ============================================================================

/// Generate the diffusion operator (Grover diffuser) for n qubits
/// 
/// The diffusion operator is: 2|s⟩⟨s| - I
/// where |s⟩ is the uniform superposition state
/// 
/// Implementation: H⊗n · (2|0⟩⟨0| - I) · H⊗n
pub fn grover_diffuser(n: usize, qubits: Option<&[usize]>) -> Vec<Gate> {
    let q: Vec<usize> = qubits.map(|qs| qs.to_vec())
        .unwrap_or_else(|| (0..n).collect());
    
    let mut gates = Vec::new();
    
    // Apply H to all qubits
    for &qubit in &q {
        gates.push(Gate::H(qubit));
    }
    
    // Apply X to all qubits
    for &qubit in &q {
        gates.push(Gate::X(qubit));
    }
    
    // Apply multi-controlled Z (or H-MCX-H on last qubit)
    if n == 1 {
        gates.push(Gate::Z(q[0]));
    } else if n == 2 {
        gates.push(Gate::CZ(q[0], q[1]));
    } else {
        // Multi-controlled Z: MCZ on all qubits
        // Implemented as H on target, MCX, H on target
        let controls: Vec<usize> = q[..n-1].to_vec();
        let target = q[n-1];
        
        gates.push(Gate::H(target));
        gates.push(Gate::MCX(controls, target));
        gates.push(Gate::H(target));
    }
    
    // Apply X to all qubits
    for &qubit in &q {
        gates.push(Gate::X(qubit));
    }
    
    // Apply H to all qubits
    for &qubit in &q {
        gates.push(Gate::H(qubit));
    }
    
    gates
}

/// Generate a complete Grover's algorithm circuit
/// 
/// # Arguments
/// * `n` - Number of qubits (search space size = 2^n)
/// * `oracle` - The oracle circuit that marks the solution states
/// * `num_iterations` - Number of Grover iterations (optimal ≈ π√N/4)
/// * `qubits` - Optional qubit indices
pub fn grover_circuit(
    n: usize,
    oracle: &[Gate],
    num_iterations: Option<usize>,
    qubits: Option<&[usize]>,
) -> Vec<Gate> {
    let q: Vec<usize> = qubits.map(|qs| qs.to_vec())
        .unwrap_or_else(|| (0..n).collect());
    
    // Optimal number of iterations for single marked state
    let iterations = num_iterations.unwrap_or_else(|| {
        let n_states = 1usize << n;
        (PI * (n_states as f64).sqrt() / 4.0).round() as usize
    });
    
    let mut gates = Vec::new();
    
    // Initialize uniform superposition
    for &qubit in &q {
        gates.push(Gate::H(qubit));
    }
    
    // Apply Grover iterations
    for _ in 0..iterations {
        // Apply oracle
        gates.extend(oracle.iter().cloned());
        
        // Apply diffuser
        gates.extend(grover_diffuser(n, Some(&q)));
    }
    
    gates
}

/// Compute optimal number of Grover iterations for k marked items
pub fn optimal_grover_iterations(n_qubits: usize, k_marked: usize) -> usize {
    let n = 1usize << n_qubits;
    if k_marked == 0 || k_marked >= n {
        return 0;
    }
    
    let theta = (k_marked as f64 / n as f64).sqrt().asin();
    let iterations = (PI / (4.0 * theta) - 0.5).round() as usize;
    iterations.max(1)
}

// ============================================================================
// AMPLITUDE AMPLIFICATION (Generalized Grover)
// ============================================================================

/// Generate amplitude amplification circuit
/// 
/// Generalizes Grover's algorithm to arbitrary initial states.
/// 
/// # Arguments
/// * `state_prep` - Circuit that prepares the initial state |ψ⟩
/// * `oracle` - Oracle marking good states
/// * `num_iterations` - Number of amplification iterations
pub fn amplitude_amplification(
    state_prep: &[Gate],
    oracle: &[Gate],
    num_iterations: usize,
) -> Vec<Gate> {
    let mut gates = Vec::new();
    
    // Prepare initial state
    gates.extend(state_prep.iter().cloned());
    
    // Compute inverse of state preparation
    let state_prep_inv: Vec<Gate> = state_prep.iter()
        .rev()
        .map(|g| g.inverse())
        .collect();
    
    for _ in 0..num_iterations {
        // Apply oracle: S_χ (marks good states with -1)
        gates.extend(oracle.iter().cloned());
        
        // Apply reflection about |ψ⟩: S_ψ = A·S_0·A†
        // Where A is state preparation and S_0 reflects about |0⟩
        gates.extend(state_prep_inv.iter().cloned());
        
        // Get qubits from state_prep for reflection
        let qubits: Vec<usize> = state_prep.iter()
            .flat_map(|g| g.qubits())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        
        // Reflect about |0⟩: X, MCZ, X
        for &q in &qubits {
            gates.push(Gate::X(q));
        }
        
        if qubits.len() == 1 {
            gates.push(Gate::Z(qubits[0]));
        } else if qubits.len() == 2 {
            gates.push(Gate::CZ(qubits[0], qubits[1]));
        } else if qubits.len() >= 3 {
            let controls: Vec<usize> = qubits[..qubits.len()-1].to_vec();
            let target = qubits[qubits.len()-1];
            gates.push(Gate::H(target));
            gates.push(Gate::MCX(controls, target));
            gates.push(Gate::H(target));
        }
        
        for &q in &qubits {
            gates.push(Gate::X(q));
        }
        
        gates.extend(state_prep.iter().cloned());
    }
    
    gates
}

// ============================================================================
// ORACLE CONSTRUCTION
// ============================================================================

/// Create a phase oracle that marks a specific computational basis state
/// 
/// Applies phase -1 to |target_state⟩, identity to others.
pub fn create_phase_oracle(n: usize, target_state: usize, qubits: Option<&[usize]>) -> Vec<Gate> {
    let q: Vec<usize> = qubits.map(|qs| qs.to_vec())
        .unwrap_or_else(|| (0..n).collect());
    
    let mut gates = Vec::new();
    
    // Apply X gates to flip bits that are 0 in target
    for i in 0..n {
        if (target_state >> i) & 1 == 0 {
            gates.push(Gate::X(q[i]));
        }
    }
    
    // Apply multi-controlled Z
    if n == 1 {
        gates.push(Gate::Z(q[0]));
    } else if n == 2 {
        gates.push(Gate::CZ(q[0], q[1]));
    } else {
        let controls: Vec<usize> = q[..n-1].to_vec();
        let target = q[n-1];
        gates.push(Gate::H(target));
        gates.push(Gate::MCX(controls.clone(), target));
        gates.push(Gate::H(target));
    }
    
    // Undo X gates
    for i in 0..n {
        if (target_state >> i) & 1 == 0 {
            gates.push(Gate::X(q[i]));
        }
    }
    
    gates
}

/// Create an oracle that marks multiple states
pub fn create_multi_state_oracle(n: usize, target_states: &[usize], qubits: Option<&[usize]>) -> Vec<Gate> {
    let mut gates = Vec::new();
    
    for &state in target_states {
        gates.extend(create_phase_oracle(n, state, qubits));
    }
    
    gates
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qft_gate_count() {
        let circuit = qft(4, None);
        // QFT on 4 qubits should have:
        // 4 H gates + (0+1+2+3) = 6 CP gates + 2 SWAP gates
        let h_count = circuit.iter().filter(|g| matches!(g, Gate::H(_))).count();
        assert_eq!(h_count, 4);
    }

    #[test]
    fn test_inverse_qft_structure() {
        let qft_gates = qft(3, None);
        let iqft_gates = inverse_qft(3, None);
        
        // iQFT should have same number of gates as QFT
        assert_eq!(qft_gates.len(), iqft_gates.len());
    }

    #[test]
    fn test_grover_optimal_iterations() {
        // For 1 marked out of 4 (2 qubits), optimal is about 1 iteration
        let iters = optimal_grover_iterations(2, 1);
        assert_eq!(iters, 1);
        
        // For 1 marked out of 256 (8 qubits), optimal is about 12 iterations
        let iters = optimal_grover_iterations(8, 1);
        assert!(iters >= 10 && iters <= 15);
    }

    #[test]
    fn test_phase_oracle() {
        // Oracle for |11⟩ on 2 qubits
        let oracle = create_phase_oracle(2, 3, None);
        // Should have CZ (no X gates since target is all 1s)
        assert!(oracle.iter().any(|g| matches!(g, Gate::CZ(_, _))));
    }
}
