//! Linear Systems and Matrix Algorithms
//!
//! This module implements algorithms for linear systems and matrix manipulation:
//! - HHL Algorithm (Harrow-Hassidim-Lloyd)
//! - Quantum Singular Value Transformation (QSVT) basics
//! - Matrix inversion (conceptual)
//!
//! ## 🎯 Why is this used?
//! Solving large-scale linear systems ($Ax = b$) is a bottleneck in scientific computing, 
//! machine learning, and engineering. Quantum algorithms like HHL offer a potential 
//! exponential speedup over classical methods. QSVT provides a unified framework for 
//! nearly all known quantum algorithms, including matrix inversion, Hamiltonian 
//! simulation, and eigenvalue search.
//!
//! ## ⚙️ How it works?
//! - **HHL**: Uses Phase Estimation to extract eigenvalues of $A$, conditionally rotates 
//!   an ancilla qubit by $1/\lambda$ to perform matrix inversion in the amplitude 
//!   domain, and then uncomputes the phases.
//! - **QSVT**: Uses block-encoding to embed a non-unitary matrix $A$ into a larger 
//!   unitary $U$. It then applies polynomial transformations $f(A)$ by interleaving 
//!   the block-encoding with project-controlled phase shifts.
//!
//! ## 📍 Where to apply this?
//! - **Scientific Computing**: Solving differential equations or optimization problems.
//! - **Quantum Machine Learning**: Implementing Quantum Least Squares or Principal Component Analysis.
//! - **Filtering**: Using QSVT as a spectral filter for matrix operators.
//!
//! ## 📊 Code Behavior
//! - **Complexity**: HHL is logarithmic in the matrix size $N$, but depends on the 
//!   condition number $\kappa$ and precision $\epsilon$.
//! - **Interaction**: Requires a "Black Box" for Hamiltonian simulation (e^(iAt)) 
//!   and state preparation for the vector $b$.
//! - **Side Effects**: Post-selection on the ancilla qubit is required to collapse 
//!    the system into the solution state.

use crate::gates::core::Gate;
use crate::algorithms::qft;

// ============================================================================
// HHL ALGORITHM (HARROW-HASSIDIM-LLOYD)
// ============================================================================

/// HHL Algorithm circuit for Ax = b
/// 
/// # Arguments
/// * `matrix_unitary` - Unitary e^(iAt) evolution circuit
/// * `clock_qubits` - Qubits used for phase estimation
/// * `target_qubits` - Qubits encoding the vector b
/// * `ancilla_qubit` - Qubit used for eigenvalue inversion
pub fn hhl_circuit(
    matrix_evolution: Vec<Gate>,
    clock_qubits: &[usize],
    _target_qubits: &[usize],
    ancilla_qubit: usize,
) -> Vec<Gate> {
    let mut gates = Vec::new();
    
    // 1. Quantum Phase Estimation (QPE) on A to find eigenvalues
    // H gates on clock register
    for &q in clock_qubits {
        gates.push(Gate::H(q));
    }
    
    // Controlled e^(iAt) evolutions
    for (i, &cq) in clock_qubits.iter().enumerate() {
        let power = 1 << i;
        for _ in 0..power {
            for g in &matrix_evolution {
                gates.extend(make_controlled(cq, g));
            }
        }
    }
    
    // Inverse QFT on clock register
    gates.extend(qft::qft(clock_qubits.len(), Some(clock_qubits)));
    // (Note: need to inverse the QFT if implemented as forward QFT)
    
    // 2. Controlled rotation of ancilla based on eigenvalues in clock register
    // |lambda>|0> -> |lambda>(sqrt(1 - (C/lambda)^2)|0> + (C/lambda)|1>)
    // For simplicity, we use a basic version with CRotations
    for (i, &cq) in clock_qubits.iter().enumerate() {
        let angle = 2.0 * (1.0 / (1 << i) as f64).asin();
        gates.push(Gate::CRY(cq, ancilla_qubit, angle));
    }
    
    // 3. Uncompute QPE
    gates.extend(qft::qft(clock_qubits.len(), Some(clock_qubits))); // Forward QFT as inverse of inverse
    
    for (i, &cq) in clock_qubits.iter().rev().enumerate() {
        let power = 1 << (clock_qubits.len() - 1 - i);
        for _ in 0..power {
            for g in &matrix_evolution {
                // In reality, this should be the inverse evolution e^(-iAt)
                gates.extend(make_controlled(cq, &g.inverse()));
            }
        }
    }
    
    for &q in clock_qubits {
        gates.push(Gate::H(q));
    }
    
    gates
}

fn make_controlled(control: usize, gate: &Gate) -> Vec<Gate> {
    match gate {
        Gate::X(t) => vec![Gate::CX(control, *t)],
        Gate::Y(t) => vec![Gate::CY(control, *t)],
        Gate::Z(t) => vec![Gate::CZ(control, *t)],
        Gate::RZ(t, theta) => vec![Gate::CRZ(control, *t, *theta)],
        Gate::H(t) => {
            // H = RY(pi/2) Z
            vec![Gate::CRY(control, *t, std::f64::consts::FRAC_PI_2), Gate::CZ(control, *t)]
        },
        _ => vec![gate.clone()], // Fallback (should decompose in real implementation)
    }
}

// ============================================================================
// QUANTUM SINGULAR VALUE TRANSFORMATION (QSVT)
// ============================================================================

/// QSVT transformation on a block-encoded matrix A
/// 
/// f(A) = U_phi_d ... U_phi_1 U_phi_0
pub fn qsvt_step(
    block_encoding: &[Gate],
    projector_zero: &[Gate],
    _phase: f64,
) -> Vec<Gate> {
    let mut gates = Vec::new();
    
    // Apply phase shield: e^(i phi (2|0><0| - I))
    gates.extend(projector_zero.iter().cloned());
    // ... rotation logic ...
    
    // Apply block encoding
    gates.extend(block_encoding.iter().cloned());
    
    gates
}
