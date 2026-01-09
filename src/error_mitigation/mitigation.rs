//! Error Mitigation Techniques
//!
//! This module provides error mitigation methods for NISQ devices:
//! - Zero Noise Extrapolation (ZNE)
//! - Probabilistic Error Cancellation (PEC)
//! - Clifford Data Regression (CDR)
//! - Symmetry verification
//!
//! ## 🎯 Why is this used?
//! In the NISQ (Noisy Intermediate-Scale Quantum) era, we do not yet have 
//! enough qubits for full error correction. Error mitigation allows us 
//! to extract useful information from noisy results by using clever 
//! classical post-processing or circuit scaling. It significantly improves 
//! the precision of expectation value estimations on real hardware.
//!
//! ## ⚙️ How it works?
//! - **Zero Noise Extrapolation (ZNE)**: Scales the noise in the circuit (e.g., 
//!   by replacing $G \rightarrow G G^\dagger G$) and extrapolates the noisy 
//!   results back to the theoretical "zero noise" limit.
//! - **Probabilistic Error Cancellation (PEC)**: Decomposes the ideal gate 
//!   into a quasi-probability distribution of noisy operations and uses 
//!   random sampling to "cancel out" noise effects on average.
//! - **Clifford Data Regression (CDR)**: Uses classically simulable Clifford 
//!   circuits to "train" a noise model and correct the results of the 
//!   target non-Clifford circuit.
//!
//! ## 📍 Where to apply this?
//! - **NISQ Experiments**: Boosting the accuracy of VQE or QAOA results.
//! - **Benchmarking**: Understanding the noise profile of a specific QPU.
//! - **State Verification**: Using post-selection to filter out states that 
//!   break known physical symmetries.
//!
//! ## 📊 Code Behavior
//! - **Complexity**: 
//!     - ZNE: Requires running $M$ circuits at different noise scales.
//!     - PEC: Causes an exponential increase in the number of shots needed.
//! - **Trade-off**: Mitigation trades classical compute time and measurement 
//!   counts for reduced bias in the final result.
//! - **Requirements**: PEC requires a precise characterization of the hardware 
//!   noise channel (Gate Set Tomography).

use std::f64::consts::PI;
use crate::gates::core::Gate;

// ============================================================================
// ZERO NOISE EXTRAPOLATION (ZNE)
// ============================================================================

/// Scale factor for ZNE noise amplification
#[derive(Clone, Debug)]
pub enum NoiseScaling {
    /// Integer folding: gate → gate·gate†·gate (repeated)
    IntegerFolding(usize),
    /// Fractional folding with probabilistic application
    FractionalFolding(f64),
    /// Pulse stretching (conceptual - requires hardware control)
    PulseStretching(f64),
}

/// Generate a noise-scaled circuit using gate folding
/// 
/// For scale factor c, each gate G becomes G(G†G)^(c-1)/2 for odd c
/// or uses probabilistic folding for non-integer c.
pub fn zne_fold_circuit(circuit: &[Gate], scale_factor: usize) -> Vec<Gate> {
    if scale_factor == 1 {
        return circuit.to_vec();
    }
    
    assert!(scale_factor % 2 == 1, "Folding scale factor must be odd");
    
    let mut folded = Vec::new();
    let folds = (scale_factor - 1) / 2;
    
    for gate in circuit {
        // Original gate
        folded.push(gate.clone());
        
        // Fold: G†G pairs
        for _ in 0..folds {
            folded.push(gate.inverse());
            folded.push(gate.clone());
        }
    }
    
    folded
}

/// Generate circuits for multiple noise scales (for extrapolation)
pub fn zne_scale_circuits(
    circuit: &[Gate], 
    scale_factors: &[usize],
) -> Vec<Vec<Gate>> {
    scale_factors.iter()
        .map(|&s| zne_fold_circuit(circuit, s))
        .collect()
}

/// Local folding for specific gates (selective noise amplification)
/// 
/// Folds only two-qubit gates to amplify their noise specifically.
pub fn zne_local_fold(circuit: &[Gate], scale_factor: usize) -> Vec<Gate> {
    if scale_factor == 1 {
        return circuit.to_vec();
    }
    
    let folds = (scale_factor.saturating_sub(1)) / 2;
    let mut folded = Vec::new();
    
    for gate in circuit {
        folded.push(gate.clone());
        
        // Only fold two-qubit gates (typically noisiest)
        if gate.qubits().len() >= 2 {
            for _ in 0..folds {
                folded.push(gate.inverse());
                folded.push(gate.clone());
            }
        }
    }
    
    folded
}

/// Richardson extrapolation for ZNE
/// 
/// Given noisy expectation values at different scale factors,
/// extrapolate to zero noise limit.
pub fn richardson_extrapolation(
    scale_factors: &[f64],
    expectation_values: &[f64],
) -> f64 {
    let n = scale_factors.len();
    assert_eq!(n, expectation_values.len());
    
    // Linear extrapolation for 2 points
    if n == 2 {
        let (c1, c2) = (scale_factors[0], scale_factors[1]);
        let (e1, e2) = (expectation_values[0], expectation_values[1]);
        return e1 + (e1 - e2) * c1 / (c2 - c1);
    }
    
    // Polynomial extrapolation for n points
    // Using Lagrange interpolation evaluated at c=0
    let mut result = 0.0;
    
    for i in 0..n {
        let mut term = expectation_values[i];
        for j in 0..n {
            if i != j {
                term *= -scale_factors[j] / (scale_factors[i] - scale_factors[j]);
            }
        }
        result += term;
    }
    
    result
}

/// Exponential extrapolation for ZNE
/// 
/// Assumes noise effects are exponential: E(c) = E(0) * exp(a*c)
pub fn exponential_extrapolation(
    scale_factors: &[f64],
    expectation_values: &[f64],
) -> f64 {
    assert!(scale_factors.len() >= 2);
    
    // Fit E(c) = A * exp(a*c) using two points
    let (c1, c2) = (scale_factors[0], scale_factors[1]);
    let (e1, e2) = (expectation_values[0], expectation_values[1]);
    
    if e1.abs() < 1e-10 || e2.abs() < 1e-10 || e1 * e2 < 0.0 {
        // Fall back to linear
        return richardson_extrapolation(scale_factors, expectation_values);
    }
    
    let a = (e2.ln() - e1.ln()) / (c2 - c1);
    let log_a_coeff = e1.ln() - a * c1;
    
    log_a_coeff.exp()
}

// ============================================================================
// PROBABILISTIC ERROR CANCELLATION (PEC)
// ============================================================================

/// Quasi-probability representation of a noisy gate
#[derive(Clone, Debug)]
pub struct QuasiProbabilityDecomposition {
    pub operations: Vec<Vec<Gate>>,
    pub coefficients: Vec<f64>,  // Can be negative
}

impl QuasiProbabilityDecomposition {
    /// Total sampling overhead (1-norm of coefficients)
    pub fn gamma(&self) -> f64 {
        self.coefficients.iter().map(|c| c.abs()).sum()
    }
}

/// Generate quasi-probability decomposition for depolarizing channel
/// 
/// For a gate G with depolarizing noise p:
/// G_noisy = (1-p)G + p/3(XGX + YGY + ZGZ) for single qubit
pub fn depolarizing_decomposition(gate: &Gate, error_rate: f64) -> QuasiProbabilityDecomposition {
    let qubits = gate.qubits();
    
    if qubits.len() == 1 {
        // Single-qubit depolarizing: I, X, Y, Z corrections
        let q = qubits[0];
        let p = error_rate;
        
        QuasiProbabilityDecomposition {
            operations: vec![
                vec![gate.clone()],
                vec![gate.clone(), Gate::X(q)],
                vec![gate.clone(), Gate::Y(q)],
                vec![gate.clone(), Gate::Z(q)],
            ],
            coefficients: vec![
                1.0 + p / (1.0 - p),
                -p / (3.0 * (1.0 - p)),
                -p / (3.0 * (1.0 - p)),
                -p / (3.0 * (1.0 - p)),
            ],
        }
    } else {
        // Two-qubit decomposition (simplified - 15 Pauli corrections)
        // Using only identity for simplicity
        QuasiProbabilityDecomposition {
            operations: vec![vec![gate.clone()]],
            coefficients: vec![1.0 / (1.0 - error_rate)],
        }
    }
}

/// Sample a circuit from quasi-probability decomposition
/// 
/// Returns (sampled_circuit, sign) where sign is ±1 for importance sampling
pub fn pec_sample_circuit(
    original_circuit: &[Gate],
    decompositions: &[QuasiProbabilityDecomposition],
    rng_values: &[f64],  // Pre-generated random values [0,1)
) -> (Vec<Gate>, f64) {
    assert_eq!(original_circuit.len(), decompositions.len());
    
    let mut sampled_circuit = Vec::new();
    let mut sign = 1.0;
    let mut rng_idx = 0;
    
    for (_gate, decomp) in original_circuit.iter().zip(decompositions.iter()) {
        let gamma = decomp.gamma();
        
        // Normalize probabilities
        let probs: Vec<f64> = decomp.coefficients.iter()
            .map(|c| c.abs() / gamma)
            .collect();
        
        // Sample from distribution
        let r = if rng_idx < rng_values.len() {
            rng_values[rng_idx]
        } else {
            0.5  // Default
        };
        rng_idx += 1;
        
        let mut cumsum = 0.0;
        let mut chosen = 0;
        for (i, &p) in probs.iter().enumerate() {
            cumsum += p;
            if r < cumsum {
                chosen = i;
                break;
            }
        }
        
        // Add sampled operations
        sampled_circuit.extend(decomp.operations[chosen].clone());
        
        // Update sign
        if decomp.coefficients[chosen] < 0.0 {
            sign *= -1.0;
        }
        sign *= gamma;
    }
    
    (sampled_circuit, sign)
}

// ============================================================================
// SYMMETRY VERIFICATION
// ============================================================================

/// Generate symmetry verification circuit
/// 
/// Adds measurements to verify that the state has expected symmetry properties.
/// For example, particle number conservation in chemistry.
pub fn symmetry_verification_circuit(
    algorithm_circuit: &[Gate],
    symmetry_operators: &[Vec<Gate>],  // Each is a unitary that should give +1
    ancilla_start: usize,
) -> Vec<Gate> {
    let mut gates = algorithm_circuit.to_vec();
    
    for (i, symmetry_op) in symmetry_operators.iter().enumerate() {
        let ancilla = ancilla_start + i;
        
        // Hadamard test for eigenvalue measurement
        gates.push(Gate::H(ancilla));
        
        // Controlled symmetry operator
        for gate in symmetry_op {
            gates.extend(make_controlled(ancilla, gate));
        }
        
        gates.push(Gate::H(ancilla));
    }
    
    gates
}

fn make_controlled(control: usize, gate: &Gate) -> Vec<Gate> {
    match gate {
        Gate::X(t) => vec![Gate::CX(control, *t)],
        Gate::Y(t) => vec![Gate::CY(control, *t)],
        Gate::Z(t) => vec![Gate::CZ(control, *t)],
        Gate::CX(c, t) => vec![Gate::CCX(control, *c, *t)],
        _ => vec![gate.clone()], // Simplified
    }
}

/// Post-selection filter for symmetry verification
/// 
/// Returns circuits with ancilla measurements and expected outcomes.
pub fn symmetry_postselection(
    algorithm_circuit: &[Gate],
    num_qubits: usize,
    symmetry_checks: &[(Vec<usize>, i32)],  // (qubits, expected_parity)
) -> Vec<Gate> {
    let mut gates = algorithm_circuit.to_vec();
    
    for (i, (qubits, _expected)) in symmetry_checks.iter().enumerate() {
        let ancilla = num_qubits + i;
        
        // Compute parity of specified qubits
        for &q in qubits {
            gates.push(Gate::CX(q, ancilla));
        }
    }
    
    gates
}

// ============================================================================
// CLIFFORD DATA REGRESSION (CDR)
// ============================================================================

/// Generate near-Clifford training circuits
/// 
/// Replaces some non-Clifford gates with Clifford gates for classical simulation.
pub fn cdr_training_circuits(
    original_circuit: &[Gate],
    num_training: usize,
    replacement_fraction: f64,
) -> Vec<Vec<Gate>> {
    let mut training = Vec::new();
    
    for seed in 0..num_training {
        let modified = cdr_replace_gates(original_circuit, replacement_fraction, seed);
        training.push(modified);
    }
    
    training
}

fn cdr_replace_gates(circuit: &[Gate], fraction: f64, seed: usize) -> Vec<Gate> {
    circuit.iter().enumerate().map(|(i, gate)| {
        // Deterministic "random" based on gate index and seed
        let should_replace = ((i + seed) as f64 / 100.0).fract() < fraction;
        
        if should_replace && is_non_clifford(gate) {
            clifford_replacement(gate)
        } else {
            gate.clone()
        }
    }).collect()
}

fn is_non_clifford(gate: &Gate) -> bool {
    match gate {
        Gate::T(_) | Gate::Tdg(_) | 
        Gate::RX(_, _) | Gate::RY(_, _) | Gate::RZ(_, _) |
        Gate::P(_, _) | Gate::U3(_, _, _, _) |
        Gate::CRX(_, _, _) | Gate::CRY(_, _, _) | Gate::CRZ(_, _, _) |
        Gate::RXX(_, _, _) | Gate::RYY(_, _, _) | Gate::RZZ(_, _, _) => true,
        _ => false,
    }
}

fn clifford_replacement(gate: &Gate) -> Gate {
    match gate {
        Gate::T(q) => Gate::S(*q),
        Gate::Tdg(q) => Gate::Sdg(*q),
        Gate::RZ(q, theta) => {
            // Round to nearest Clifford angle
            let cliff_angle = (theta / (PI / 2.0)).round() * (PI / 2.0);
            if cliff_angle.abs() < 1e-10 { Gate::H(*q) } // Identity-ish
            else if (cliff_angle - PI / 2.0).abs() < 1e-10 { Gate::S(*q) }
            else if (cliff_angle - PI).abs() < 1e-10 { Gate::Z(*q) }
            else { Gate::Sdg(*q) }
        }
        Gate::RX(q, _) => Gate::X(*q),
        Gate::RY(q, _) => Gate::Y(*q),
        _ => gate.clone(),
    }
}

/// Linear regression model for CDR
pub fn cdr_linear_fit(
    noisy_training: &[f64],
    ideal_training: &[f64],
    noisy_target: f64,
) -> f64 {
    assert_eq!(noisy_training.len(), ideal_training.len());
    
    let n = noisy_training.len() as f64;
    
    // Simple linear regression: ideal = a + b * noisy
    let sum_x: f64 = noisy_training.iter().sum();
    let sum_y: f64 = ideal_training.iter().sum();
    let sum_xy: f64 = noisy_training.iter()
        .zip(ideal_training.iter())
        .map(|(x, y)| x * y)
        .sum();
    let sum_xx: f64 = noisy_training.iter().map(|x| x * x).sum();
    
    let b = (n * sum_xy - sum_x * sum_y) / (n * sum_xx - sum_x * sum_x);
    let a = (sum_y - b * sum_x) / n;
    
    a + b * noisy_target
}

// ============================================================================
// VIRTUAL DISTILLATION
// ============================================================================

/// Generate virtual distillation circuit
/// 
/// Uses M copies of the noisy state to purify:
/// Tr(ρ^M) / Tr(ρ^(M-1)) as M→∞ approaches the largest eigenvalue
pub fn virtual_distillation_circuit(
    state_prep: &[Gate],
    num_copies: usize,
    qubits_per_copy: usize,
) -> Vec<Gate> {
    let mut gates = Vec::new();
    
    // Prepare each copy
    for copy in 0..num_copies {
        let offset = copy * qubits_per_copy;
        for gate in state_prep {
            gates.push(shift_gate(gate, offset));
        }
    }
    
    // Apply cyclic SWAP measurement
    let ancilla = num_copies * qubits_per_copy;
    gates.push(Gate::H(ancilla));
    
    // Controlled cyclic permutation between copies
    for q in 0..qubits_per_copy {
        for copy in 0..num_copies - 1 {
            let q1 = copy * qubits_per_copy + q;
            let q2 = (copy + 1) * qubits_per_copy + q;
            gates.push(Gate::CSWAP(ancilla, q1, q2));
        }
    }
    
    gates.push(Gate::H(ancilla));
    
    gates
}

fn shift_gate(gate: &Gate, offset: usize) -> Gate {
    match gate {
        Gate::X(q) => Gate::X(q + offset),
        Gate::Y(q) => Gate::Y(q + offset),
        Gate::Z(q) => Gate::Z(q + offset),
        Gate::H(q) => Gate::H(q + offset),
        Gate::S(q) => Gate::S(q + offset),
        Gate::Sdg(q) => Gate::Sdg(q + offset),
        Gate::T(q) => Gate::T(q + offset),
        Gate::Tdg(q) => Gate::Tdg(q + offset),
        Gate::RX(q, t) => Gate::RX(q + offset, *t),
        Gate::RY(q, t) => Gate::RY(q + offset, *t),
        Gate::RZ(q, t) => Gate::RZ(q + offset, *t),
        Gate::CX(c, t) => Gate::CX(c + offset, t + offset),
        Gate::CZ(c, t) => Gate::CZ(c + offset, t + offset),
        _ => gate.clone(),
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zne_folding() {
        let circuit = vec![Gate::H(0), Gate::CX(0, 1)];
        
        // Scale factor 3: G → G G† G
        let folded = zne_fold_circuit(&circuit, 3);
        assert_eq!(folded.len(), 6); // 2 gates × 3 copies
    }

    #[test]
    fn test_richardson_extrapolation() {
        // Linear noise model: E(c) = 1.0 - 0.1*c
        // At c=1: 0.9, at c=3: 0.7, at c=5: 0.5
        // Extrapolate to c=0: should be 1.0
        
        let scales = vec![1.0, 3.0, 5.0];
        let values = vec![0.9, 0.7, 0.5];
        
        let result = richardson_extrapolation(&scales, &values);
        assert!((result - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_depolarizing_decomposition() {
        let gate = Gate::X(0);
        let decomp = depolarizing_decomposition(&gate, 0.01);
        
        // Should have 4 operations for single qubit
        assert_eq!(decomp.operations.len(), 4);
        
        // Gamma should be > 1 for non-zero error
        assert!(decomp.gamma() > 1.0);
    }

    #[test]
    fn test_cdr_training_circuits() {
        let circuit = vec![Gate::T(0), Gate::CX(0, 1), Gate::T(1)];
        
        let training = cdr_training_circuits(&circuit, 5, 0.5);
        assert_eq!(training.len(), 5);
    }
}
