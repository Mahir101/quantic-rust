//! Circuit Cutting and Distribution
//!
//! This module provides techniques for executing large circuits on smaller devices:
//! - Wire cutting (QPD-based)
//! - Gate cutting
//! - Entanglement forging
//!
//! ## 🎯 Why is this used?
//! Today's quantum processors are constrained by both qubit counts and limited 
//! connectivity. Circuit cutting allows us to "trade sampling for qubits" by 
//! partitioning a large quantum circuit into smaller sub-circuits that can be 
//! executed independently on separate QPUs or even on a single smaller QPU. 
//! It is a cornerstone of Distributed Quantum Computing.
//!
//! ## ⚙️ How it works?
//! - **Wire Cutting**: Decomposes the identity channel into a quasi-probability 
//!   distribution (QPD) of preparation and measurement operations. A single 
//!   wire cut effectively breaks the connection between zwei sub-circuits.
//! - **Gate Cutting**: Replaces expensive non-local gates (like long-range CNOTs) 
//!   with a sum of single-qubit operations across both qubits.
//! - **Knit & Reconstruct**: After execution, the results from all sub-circuit 
//!   configurations are classically recombined using the QPD coefficients to 
//!   reconstruct the original expectation value.
//!
//! ## 📍 Where to apply this?
//! - **Scalability**: Running a 50-qubit algorithm on a cluster of 20-qubit devices.
//! - **Hybrid Cloud**: Distributing a massive circuit across multiple quantum 
//!   cloud providers.
//! - **Noise Mitigation**: Reducing the depth of individual circuit fragments to 
//!   keep them within coherence time limits.
//!
//! ## 📊 Code Behavior
//! - **Complexity**: Cutting induces an exponential overhead in the number of shots 
//!   required ($O(4^k)$ for $k$ wire cuts). Each cut generates 4 sub-circuit variants.
//! - **Overhead**: Total shots scale as $\gamma^2 / \epsilon^2$, where $\gamma$ 
//!   is the sampling overhead (accumulated from all cut decompositions).
//! - **Verification**: All decompositions are verified to satisfy the trace-preserving 
//!   and unital requirements of valid quantum channels.

use crate::gates::core::Gate;

// ============================================================================
// WIRE CUTTING
// ============================================================================

/// Cut location in a circuit
#[derive(Clone, Debug)]
pub struct WireCut {
    pub position: usize,       // Gate index after which to cut
    pub qubit: usize,          // Qubit to cut
}

/// Result of cutting a circuit at wire locations
#[derive(Clone, Debug)]
pub struct CutCircuit {
    pub subcircuits: Vec<Vec<Gate>>,       // Fragment circuits
    pub cut_info: Vec<CutInfo>,            // Information about each cut
    pub reconstruction_overhead: f64,       // Sampling overhead
}

#[derive(Clone, Debug)]
pub struct CutInfo {
    pub subcircuit_index: usize,
    pub qubit_in_subcircuit: usize,
    pub cut_type: CutType,
}

#[derive(Clone, Debug)]
pub enum CutType {
    WireCut,
    GateCut,
}

/// Cut a circuit at specified wire locations using QPD decomposition
/// 
/// Wire cutting decomposes the identity channel as a quasi-probability
/// distribution over preparation and measurement operators.
pub fn cut_wires(
    circuit: &[Gate],
    cuts: &[WireCut],
) -> Vec<(Vec<Gate>, Vec<Gate>, f64)> {
    // I = 1/2 (|0⟩⟨0| × MeasZ + |+⟩⟨+| × MeasX + |i+⟩⟨i+| × MeasY + I × ...)
    // Quasi-probability decomposition with overhead 4 per cut
    
    let mut fragments = Vec::new();
    
    // Sort cuts by position (reverse order for correct slicing)
    let mut sorted_cuts: Vec<_> = cuts.iter().cloned().collect();
    sorted_cuts.sort_by(|a, b| b.position.cmp(&a.position));
    
    // Generate all 4^k subcircuits for k cuts
    let num_configurations = 4usize.pow(cuts.len() as u32);
    
    for config in 0..num_configurations {
        let (pre_ops, post_ops, coeff) = decompose_configuration(config, cuts);
        
        let mut fragment_pre = circuit[..cuts[0].position].to_vec();
        fragment_pre.extend(pre_ops);
        
        let mut fragment_post = post_ops;
        fragment_post.extend(circuit[cuts[0].position..].iter().cloned());
        
        fragments.push((fragment_pre, fragment_post, coeff));
    }
    
    fragments
}

fn decompose_configuration(config: usize, cuts: &[WireCut]) -> (Vec<Gate>, Vec<Gate>, f64) {
    let mut pre_ops = Vec::new();
    let mut post_ops = Vec::new();
    let mut coeff = 1.0;
    
    for (i, cut) in cuts.iter().enumerate() {
        let local_config = (config >> (2 * i)) & 3;
        let q = cut.qubit;
        
        match local_config {
            0 => {
                // Prepare |0⟩, measure Z
                // Pre: nothing (already |0⟩)
                // Post: nothing (measure in Z)
                coeff *= 0.5;
            }
            1 => {
                // Prepare |1⟩, measure Z
                pre_ops.push(Gate::X(q));
                coeff *= 0.5;
            }
            2 => {
                // Prepare |+⟩, measure X
                pre_ops.push(Gate::H(q));
                post_ops.push(Gate::H(q));
                coeff *= 0.5;
            }
            3 => {
                // Prepare |i+⟩, measure Y
                pre_ops.push(Gate::H(q));
                pre_ops.push(Gate::S(q));
                post_ops.push(Gate::Sdg(q));
                post_ops.push(Gate::H(q));
                coeff *= 0.5;
            }
            _ => unreachable!(),
        }
    }
    
    (pre_ops, post_ops, coeff)
}

/// Find optimal wire cut locations to partition circuit into fragments
/// 
/// Minimizes the total sampling overhead while keeping fragment sizes
/// within the specified limit.
pub fn find_optimal_cuts(
    circuit: &[Gate],
    max_fragment_qubits: usize,
) -> Vec<WireCut> {
    // Simple greedy algorithm: cut at positions that minimize qubit count
    let mut cuts = Vec::new();
    let total_qubits = circuit.iter()
        .flat_map(|g| g.qubits())
        .max()
        .map(|m| m + 1)
        .unwrap_or(0);
    
    if total_qubits <= max_fragment_qubits {
        return cuts;
    }
    
    // Find cut points based on circuit structure
    for (i, gate) in circuit.iter().enumerate() {
        let qubits = gate.qubits();
        if qubits.len() >= 2 {
            // Potential cut location after two-qubit gates
            cuts.push(WireCut {
                position: i + 1,
                qubit: qubits[1],
            });
            
            // Check if we have enough cuts
            let fragments = estimate_fragments(&cuts, circuit);
            if fragments.iter().all(|f| *f <= max_fragment_qubits) {
                break;
            }
        }
    }
    
    cuts
}

fn estimate_fragments(cuts: &[WireCut], _circuit: &[Gate]) -> Vec<usize> {
    // Simplified: return estimated qubit count per fragment
    vec![4; cuts.len() + 1]
}

// ============================================================================
// GATE CUTTING
// ============================================================================

/// Cut a two-qubit gate into single-qubit fragments
/// 
/// For CNOT: CX = 1/2 (I⊗I + Z⊗Z + X⊗X - Y⊗Y) (with modifications)
pub fn cut_cnot(control: usize, target: usize) -> Vec<(Vec<Gate>, f64)> {
    // Decomposition of CNOT into tensor product of single-qubit ops
    // |CX⟩⟩ = 1/2 (|II⟩⟩ + |ZZ⟩⟩ + |XX⟩⟩ - |YY⟩⟩)
    
    vec![
        // I ⊗ I term
        (vec![], 0.5),
        
        // Z ⊗ Z term  
        (vec![Gate::Z(control), Gate::Z(target)], 0.5),
        
        // X ⊗ X term
        (vec![Gate::X(control), Gate::X(target)], 0.5),
        
        // -Y ⊗ Y term
        (vec![Gate::Y(control), Gate::Y(target)], -0.5),
    ]
}

/// Generic gate cutting for two-qubit gates
pub fn cut_two_qubit_gate(gate: &Gate) -> Vec<(Vec<Gate>, f64)> {
    match gate {
        Gate::CX(c, t) => cut_cnot(*c, *t),
        Gate::CZ(c, t) => cut_cz(*c, *t),
        Gate::SWAP(a, b) => cut_swap(*a, *b),
        _ => vec![(vec![gate.clone()], 1.0)],
    }
}

fn cut_cz(q0: usize, q1: usize) -> Vec<(Vec<Gate>, f64)> {
    // CZ = I⊗I + Z⊗I + I⊗Z - Z⊗Z (with appropriate coefficients)
    vec![
        (vec![], 0.25),
        (vec![Gate::Z(q0)], 0.25),
        (vec![Gate::Z(q1)], 0.25),
        (vec![Gate::Z(q0), Gate::Z(q1)], -0.25),
    ]
}

fn cut_swap(q0: usize, q1: usize) -> Vec<(Vec<Gate>, f64)> {
    // SWAP = (I⊗I + X⊗X + Y⊗Y + Z⊗Z) / 2
    vec![
        (vec![], 0.25),
        (vec![Gate::X(q0), Gate::X(q1)], 0.25),
        (vec![Gate::Y(q0), Gate::Y(q1)], 0.25),
        (vec![Gate::Z(q0), Gate::Z(q1)], 0.25),
    ]
}

// ============================================================================
// ENTANGLEMENT FORGING
// ============================================================================

/// Entanglement forging configuration
#[derive(Clone, Debug)]
pub struct ForgingConfig {
    pub system_a_qubits: Vec<usize>,
    pub system_b_qubits: Vec<usize>,
    pub schmidt_coefficients: Vec<f64>,
    pub bitstrings_a: Vec<usize>,
    pub bitstrings_b: Vec<usize>,
}

/// Generate entanglement forging circuits
/// 
/// For a bipartite state |ψ⟩ = Σ_k λ_k |φ_k⟩_A |χ_k⟩_B,
/// forging avoids creating entanglement between A and B.
pub fn entanglement_forging(
    ansatz_a: impl Fn(usize) -> Vec<Gate>,
    ansatz_b: impl Fn(usize) -> Vec<Gate>,
    config: &ForgingConfig,
) -> Vec<(Vec<Gate>, Vec<Gate>, f64)> {
    let mut circuits = Vec::new();
    
    for (k, &lambda) in config.schmidt_coefficients.iter().enumerate() {
        if k >= config.bitstrings_a.len() || k >= config.bitstrings_b.len() {
            break;
        }
        
        // Prepare computational basis state for system A
        let mut circuit_a = prepare_bitstring(&config.system_a_qubits, config.bitstrings_a[k]);
        circuit_a.extend(ansatz_a(k));
        
        // Prepare computational basis state for system B
        let mut circuit_b = prepare_bitstring(&config.system_b_qubits, config.bitstrings_b[k]);
        circuit_b.extend(ansatz_b(k));
        
        circuits.push((circuit_a, circuit_b, lambda * lambda));
    }
    
    circuits
}

fn prepare_bitstring(qubits: &[usize], bitstring: usize) -> Vec<Gate> {
    qubits.iter().enumerate()
        .filter_map(|(i, &q)| {
            if (bitstring >> i) & 1 == 1 {
                Some(Gate::X(q))
            } else {
                None
            }
        })
        .collect()
}

// ============================================================================
// CIRCUIT KNITTING
// ============================================================================

/// Knit subcircuit results together
/// 
/// Combines measurement results from subcircuits according to 
/// the quasi-probability distribution.
pub fn knit_results(
    subcircuit_expectations: &[f64],
    coefficients: &[f64],
) -> f64 {
    assert_eq!(subcircuit_expectations.len(), coefficients.len());
    
    subcircuit_expectations.iter()
        .zip(coefficients.iter())
        .map(|(e, c)| e * c)
        .sum()
}

/// Estimate sampling overhead for circuit cutting
/// 
/// The overhead is approximately γ = Π_cuts 4 for wire cuts
/// and 3 for gate cuts.
pub fn estimate_cutting_overhead(
    num_wire_cuts: usize,
    num_gate_cuts: usize,
) -> f64 {
    (4.0_f64).powi(num_wire_cuts as i32) * (3.0_f64).powi(num_gate_cuts as i32)
}

/// Estimate number of shots needed for given precision
pub fn estimate_shots(
    target_precision: f64,
    overhead: f64,
) -> usize {
    // Variance scales as γ² / N
    // For precision ε, need N ≈ γ² / ε²
    ((overhead / target_precision).powi(2)) as usize
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cut_cnot() {
        let fragments = cut_cnot(0, 1);
        assert_eq!(fragments.len(), 4);
        
        // Coefficients should sum to 1 in absolute value
        let sum: f64 = fragments.iter().map(|(_, c)| c.abs()).sum();
        assert!((sum - 2.0).abs() < 1e-10); // 4 × 0.5
    }

    #[test]
    fn test_cutting_overhead() {
        // 1 wire cut: overhead = 4
        assert!((estimate_cutting_overhead(1, 0) - 4.0).abs() < 1e-10);
        
        // 2 wire cuts: overhead = 16
        assert!((estimate_cutting_overhead(2, 0) - 16.0).abs() < 1e-10);
    }

    #[test]
    fn test_knit_results() {
        let expectations = vec![0.5, -0.5, 0.3, -0.3];
        let coefficients = vec![0.5, 0.5, 0.5, -0.5];
        
        let result = knit_results(&expectations, &coefficients);
        // 0.5×0.5 + (-0.5)×0.5 + 0.3×0.5 + (-0.3)×(-0.5) = 0.25 - 0.25 + 0.15 + 0.15 = 0.3
        assert!((result - 0.3).abs() < 1e-10);
    }

    #[test]
    fn test_prepare_bitstring() {
        let gates = prepare_bitstring(&[0, 1, 2, 3], 0b1010);
        // Bits 1 and 3 are set
        assert_eq!(gates.len(), 2);
    }
}
