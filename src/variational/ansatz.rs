//! Variational Quantum Algorithms
//!
//! This module provides circuit constructions for variational algorithms:
//! - Hardware Efficient Ansatz (HEA)
//! - UCCSD Ansatz
//! - QAOA circuits
//! - Parameter shift gradient computation
//!
//! ## 🎯 Why is this used?
//! Variational algorithms (like VQE and QAOA) are the primary candidates for 
//! achieving quantum advantage on NISQ hardware. This module provides the 
//! "ansatz" (parameterized circuit templates) that act as a searchable 
//! space for optimization. These are used to find ground state energies 
//! in chemistry or solve combinatorial optimization problems in logistics.
//!
//! ## ⚙️ How it works?
//! - **HEA**: Uses a repeating pattern of single-qubit rotations and entangling 
//!   layers (like CZ or CNOT) that are easy to execute on specific hardware.
//! - **UCCSD**: A chemically-inspired ansatz that uses Unitary Coupled Cluster 
//!   theory to model electron correlations in molecules.
//! - **QAOA**: Interleaves a problem-specific Hamiltonian (cost) with a 
//!   transverse-field Hamiltonian (mixer) to drive the system toward the 
//!   optimal solution of an Ising problem.
//! - **Parameter-Shift Rule**: Provides an exact analytical method to calculate 
//!   gradients of the expectation value with respect to circuit parameters.
//!
//! ## 📍 Where to apply this?
//! - **Quantum Chemistry**: Solving for molecular properties (VQE).
//! - **Optimization**: Solving Max-Cut, TSP, or financial portfolio optimization (QAOA).
//! - **QML**: Training quantum neural networks.
//!
//! ## 📊 Code Behavior
//! - **Performance**: Circuit generation is $O(L \times N)$ where $L$ is 
//!   the number of layers and $N$ is the number of qubits.
//! - **Gradient Cost**: Parameter shift requires $2P$ circuit executions 
//!   to find the gradient for $P$ parameters.
//! - **Barren Plateaus**: Users should be aware that high-depth HEA ansätze 
//!   can suffer from vanishing gradients.

use std::f64::consts::PI;
use crate::gates::core::Gate;

// ============================================================================
// HARDWARE EFFICIENT ANSATZ (HEA)
// ============================================================================

/// Layer type for hardware efficient ansatz
#[derive(Clone, Debug)]
pub enum HEALayerType {
    /// RY-RZ rotations on each qubit
    RyRz,
    /// RX-RY-RZ rotations on each qubit
    RxRyRz,
    /// Only RY rotations
    RyOnly,
    /// Only RZ rotations
    RzOnly,
}

/// Entangling pattern for HEA
#[derive(Clone, Debug)]
pub enum EntanglingPattern {
    /// Linear chain: CNOT(0,1), CNOT(1,2), ...
    Linear,
    /// Full connectivity: all pairs
    Full,
    /// Circular: linear + CNOT(n-1, 0)
    Circular,
    /// Alternating layers
    Alternating,
}

/// Configuration for Hardware Efficient Ansatz
#[derive(Clone, Debug)]
pub struct HEAConfig {
    pub num_qubits: usize,
    pub num_layers: usize,
    pub rotation_type: HEALayerType,
    pub entangling_pattern: EntanglingPattern,
    pub include_initial_layer: bool,
}

impl Default for HEAConfig {
    fn default() -> Self {
        HEAConfig {
            num_qubits: 4,
            num_layers: 2,
            rotation_type: HEALayerType::RyRz,
            entangling_pattern: EntanglingPattern::Linear,
            include_initial_layer: true,
        }
    }
}

/// Generate Hardware Efficient Ansatz circuit
/// 
/// Structure: [initial rotations] (rotation layer + entangling layer) × depth
/// 
/// # Arguments
/// * `config` - Ansatz configuration
/// * `parameters` - Rotation angles (must match required parameter count)
pub fn hardware_efficient_ansatz(config: &HEAConfig, parameters: &[f64]) -> Vec<Gate> {
    let n = config.num_qubits;
    let params_per_rotation = match config.rotation_type {
        HEALayerType::RyRz => 2,
        HEALayerType::RxRyRz => 3,
        HEALayerType::RyOnly | HEALayerType::RzOnly => 1,
    };
    
    let initial_params = if config.include_initial_layer { n * params_per_rotation } else { 0 };
    let layer_params = n * params_per_rotation;
    let expected_params = initial_params + config.num_layers * layer_params;
    
    assert_eq!(parameters.len(), expected_params, 
        "Expected {} parameters, got {}", expected_params, parameters.len());
    
    let mut gates = Vec::new();
    let mut param_idx = 0;
    
    // Initial rotation layer
    if config.include_initial_layer {
        gates.extend(rotation_layer(n, &config.rotation_type, &parameters[param_idx..param_idx + layer_params]));
        param_idx += layer_params;
    }
    
    // Repeated ansatz layers
    for _ in 0..config.num_layers {
        gates.extend(rotation_layer(n, &config.rotation_type, &parameters[param_idx..param_idx + layer_params]));
        param_idx += layer_params;
        
        gates.extend(entangling_layer(n, &config.entangling_pattern));
    }
    
    gates
}

fn rotation_layer(n: usize, rotation_type: &HEALayerType, params: &[f64]) -> Vec<Gate> {
    let mut gates = Vec::new();
    let mut param_idx = 0;
    
    for q in 0..n {
        match rotation_type {
            HEALayerType::RyRz => {
                gates.push(Gate::RY(q, params[param_idx]));
                gates.push(Gate::RZ(q, params[param_idx + 1]));
                param_idx += 2;
            }
            HEALayerType::RxRyRz => {
                gates.push(Gate::RX(q, params[param_idx]));
                gates.push(Gate::RY(q, params[param_idx + 1]));
                gates.push(Gate::RZ(q, params[param_idx + 2]));
                param_idx += 3;
            }
            HEALayerType::RyOnly => {
                gates.push(Gate::RY(q, params[param_idx]));
                param_idx += 1;
            }
            HEALayerType::RzOnly => {
                gates.push(Gate::RZ(q, params[param_idx]));
                param_idx += 1;
            }
        }
    }
    
    gates
}

fn entangling_layer(n: usize, pattern: &EntanglingPattern) -> Vec<Gate> {
    let mut gates = Vec::new();
    
    match pattern {
        EntanglingPattern::Linear => {
            for i in 0..n-1 {
                gates.push(Gate::CX(i, i + 1));
            }
        }
        EntanglingPattern::Circular => {
            for i in 0..n-1 {
                gates.push(Gate::CX(i, i + 1));
            }
            if n > 1 {
                gates.push(Gate::CX(n - 1, 0));
            }
        }
        EntanglingPattern::Full => {
            for i in 0..n {
                for j in i+1..n {
                    gates.push(Gate::CX(i, j));
                }
            }
        }
        EntanglingPattern::Alternating => {
            // Even layer: 0-1, 2-3, ...
            for i in (0..n-1).step_by(2) {
                gates.push(Gate::CX(i, i + 1));
            }
            // Odd layer: 1-2, 3-4, ...
            for i in (1..n-1).step_by(2) {
                gates.push(Gate::CX(i, i + 1));
            }
        }
    }
    
    gates
}

// ============================================================================
// UCCSD ANSATZ
// ============================================================================

/// Single excitation operator exp(θ(a†_p a_q - a†_q a_p))
/// 
/// For spin-orbital indices p, q (q < p assumed)
pub fn single_excitation(p: usize, q: usize, theta: f64) -> Vec<Gate> {
    // Jordan-Wigner transformation:
    // a†_p a_q → (X_p - iY_p)/2 × Z_{p-1}×...×Z_{q+1} × (X_q + iY_q)/2
    
    let mut gates = Vec::new();
    
    // exp(iθ/2 (XY - YX)) between qubits p and q with Z string
    // Simplified: use the circuit for Pauli gadget
    
    gates.push(Gate::H(p));
    gates.push(Gate::H(q));
    
    // Z string between q and p
    for k in q+1..p {
        gates.push(Gate::CX(k, p));
    }
    gates.push(Gate::CX(q, p));
    
    gates.push(Gate::RZ(p, theta));
    
    gates.push(Gate::CX(q, p));
    for k in (q+1..p).rev() {
        gates.push(Gate::CX(k, p));
    }
    
    gates.push(Gate::H(q));
    gates.push(Gate::H(p));
    
    gates
}

/// Double excitation operator exp(θ(a†_p a†_q a_r a_s - h.c.))
/// 
/// This is the key component of UCCSD
pub fn double_excitation(p: usize, q: usize, r: usize, s: usize, theta: f64) -> Vec<Gate> {
    // Double excitation requires 8 Pauli exponentials
    // Simplified implementation using ladder approach
    
    let mut gates = Vec::new();
    
    // Use the fermionic swap network approach
    // This is a simplified version - full UCCSD has more complex circuit
    
    let angle = theta / 8.0;
    
    // The 8 terms in the double excitation
    let terms: [(bool, bool, bool, bool); 8] = [
        (false, false, false, false), // XXXX
        (false, false, true, true),   // XXYY
        (false, true, false, true),   // XYXY
        (false, true, true, false),   // XYYX
        (true, false, false, true),   // YXXY
        (true, false, true, false),   // YXYX
        (true, true, false, false),   // YYXX
        (true, true, true, true),     // YYYY
    ];
    
    for (i, (py, qy, ry, sy)) in terms.iter().enumerate() {
        let sign = if i % 2 == 0 { 1.0 } else { -1.0 };
        
        // Basis change
        if *py { gates.push(Gate::Sdg(p)); }
        gates.push(Gate::H(p));
        
        if *qy { gates.push(Gate::Sdg(q)); }
        gates.push(Gate::H(q));
        
        if *ry { gates.push(Gate::Sdg(r)); }
        gates.push(Gate::H(r));
        
        if *sy { gates.push(Gate::Sdg(s)); }
        gates.push(Gate::H(s));
        
        // CNOT ladder
        gates.push(Gate::CX(p, q));
        gates.push(Gate::CX(q, r));
        gates.push(Gate::CX(r, s));
        
        // Rotation
        gates.push(Gate::RZ(s, sign * angle));
        
        // Undo CNOT ladder
        gates.push(Gate::CX(r, s));
        gates.push(Gate::CX(q, r));
        gates.push(Gate::CX(p, q));
        
        // Undo basis change
        gates.push(Gate::H(s));
        if *sy { gates.push(Gate::S(s)); }
        
        gates.push(Gate::H(r));
        if *ry { gates.push(Gate::S(r)); }
        
        gates.push(Gate::H(q));
        if *qy { gates.push(Gate::S(q)); }
        
        gates.push(Gate::H(p));
        if *py { gates.push(Gate::S(p)); }
    }
    
    gates
}

/// Generate full UCCSD ansatz for a molecular system
/// 
/// # Arguments
/// * `num_orbitals` - Number of spatial orbitals
/// * `num_electrons` - Number of electrons
/// * `single_amplitudes` - θ parameters for single excitations
/// * `double_amplitudes` - θ parameters for double excitations
pub fn uccsd_ansatz(
    num_orbitals: usize,
    num_electrons: usize,
    single_amplitudes: &[f64],
    double_amplitudes: &[f64],
) -> Vec<Gate> {
    let num_qubits = 2 * num_orbitals; // 2 spin orbitals per spatial orbital
    
    let mut gates = Vec::new();
    
    // Prepare Hartree-Fock reference state
    for i in 0..num_electrons {
        gates.push(Gate::X(i));
    }
    
    let mut single_idx = 0;
    let mut double_idx = 0;
    
    // Single excitations (occupied → virtual)
    for i in 0..num_electrons {
        for a in num_electrons..num_qubits {
            if single_idx < single_amplitudes.len() {
                gates.extend(single_excitation(a, i, single_amplitudes[single_idx]));
                single_idx += 1;
            }
        }
    }
    
    // Double excitations
    for i in 0..num_electrons {
        for j in i+1..num_electrons {
            for a in num_electrons..num_qubits {
                for b in a+1..num_qubits {
                    if double_idx < double_amplitudes.len() {
                        gates.extend(double_excitation(a, b, j, i, double_amplitudes[double_idx]));
                        double_idx += 1;
                    }
                }
            }
        }
    }
    
    gates
}

// ============================================================================
// QAOA (QUANTUM APPROXIMATE OPTIMIZATION ALGORITHM)
// ============================================================================

/// QAOA Mixer Hamiltonian layer
/// 
/// Applies exp(-iβ Σ X_j) = Π RX(2β)_j
pub fn qaoa_mixer(num_qubits: usize, beta: f64) -> Vec<Gate> {
    (0..num_qubits)
        .map(|q| Gate::RX(q, 2.0 * beta))
        .collect()
}

/// QAOA Cost Hamiltonian for MaxCut
/// 
/// H_C = Σ_{(i,j)∈E} (1 - Z_i Z_j) / 2
/// exp(-iγ H_C) = Π RZZ(γ)_{ij}
/// 
/// # Arguments
/// * `edges` - List of edges (pairs of vertex indices)
/// * `gamma` - QAOA angle parameter
pub fn qaoa_cost_maxcut(edges: &[(usize, usize)], gamma: f64) -> Vec<Gate> {
    let mut gates = Vec::new();
    
    for &(i, j) in edges {
        gates.push(Gate::RZZ(i, j, gamma));
    }
    
    gates
}

/// QAOA Cost Hamiltonian for weighted MaxCut
pub fn qaoa_cost_weighted_maxcut(edges: &[(usize, usize, f64)], gamma: f64) -> Vec<Gate> {
    let mut gates = Vec::new();
    
    for &(i, j, weight) in edges {
        gates.push(Gate::RZZ(i, j, gamma * weight));
    }
    
    gates
}

/// Full QAOA circuit for MaxCut
/// 
/// # Arguments
/// * `num_qubits` - Number of vertices
/// * `edges` - Graph edges
/// * `gammas` - Cost layer parameters
/// * `betas` - Mixer layer parameters
pub fn qaoa_maxcut(
    num_qubits: usize,
    edges: &[(usize, usize)],
    gammas: &[f64],
    betas: &[f64],
) -> Vec<Gate> {
    assert_eq!(gammas.len(), betas.len(), 
        "Must have same number of gamma and beta parameters");
    
    let p = gammas.len();
    let mut gates = Vec::new();
    
    // Initial uniform superposition
    for q in 0..num_qubits {
        gates.push(Gate::H(q));
    }
    
    // QAOA layers
    for layer in 0..p {
        gates.extend(qaoa_cost_maxcut(edges, gammas[layer]));
        gates.extend(qaoa_mixer(num_qubits, betas[layer]));
    }
    
    gates
}

/// QAOA for general Ising Hamiltonians
/// 
/// H = Σ_i h_i Z_i + Σ_{i<j} J_{ij} Z_i Z_j
pub fn qaoa_ising(
    h: &[f64],           // Local fields
    j: &[(usize, usize, f64)],  // Couplings (i, j, J_ij)
    gammas: &[f64],
    betas: &[f64],
) -> Vec<Gate> {
    let num_qubits = h.len();
    let mut gates = Vec::new();
    
    // Initial state
    for q in 0..num_qubits {
        gates.push(Gate::H(q));
    }
    
    for layer in 0..gammas.len() {
        // Local field terms
        for (q, &field) in h.iter().enumerate() {
            if field.abs() > 1e-10 {
                gates.push(Gate::RZ(q, 2.0 * gammas[layer] * field));
            }
        }
        
        // Coupling terms
        for &(i, j_idx, coupling) in j {
            gates.push(Gate::RZZ(i, j_idx, 2.0 * gammas[layer] * coupling));
        }
        
        // Mixer
        gates.extend(qaoa_mixer(num_qubits, betas[layer]));
    }
    
    gates
}

// ============================================================================
// PARAMETER SHIFT RULE
// ============================================================================

/// Generate circuits for parameter shift gradient computation
/// 
/// For f(θ) = ⟨ψ(θ)|H|ψ(θ)⟩, the gradient is:
/// df/dθ = (f(θ + π/2) - f(θ - π/2)) / 2
/// 
/// Returns two circuits: one with θ + π/2, one with θ - π/2
pub fn parameter_shift_circuits(
    circuit_generator: impl Fn(&[f64]) -> Vec<Gate>,
    parameters: &[f64],
    param_index: usize,
) -> (Vec<Gate>, Vec<Gate>) {
    let shift = PI / 2.0;
    
    let mut params_plus = parameters.to_vec();
    params_plus[param_index] += shift;
    
    let mut params_minus = parameters.to_vec();
    params_minus[param_index] -= shift;
    
    (circuit_generator(&params_plus), circuit_generator(&params_minus))
}

/// Generate all parameter shift circuits for gradient computation
pub fn all_parameter_shift_circuits(
    circuit_generator: impl Fn(&[f64]) -> Vec<Gate>,
    parameters: &[f64],
) -> Vec<(Vec<Gate>, Vec<Gate>)> {
    (0..parameters.len())
        .map(|i| parameter_shift_circuits(&circuit_generator, parameters, i))
        .collect()
}

// ============================================================================
// SWAP TEST
// ============================================================================

/// SWAP test circuit for computing state overlap |⟨ψ|φ⟩|²
/// 
/// Uses an ancilla qubit to compute the overlap between states
/// on two registers.
/// 
/// # Arguments
/// * `ancilla` - Control qubit for the test
/// * `register1` - First state register qubits
/// * `register2` - Second state register qubits
pub fn swap_test(
    ancilla: usize,
    register1: &[usize],
    register2: &[usize],
) -> Vec<Gate> {
    assert_eq!(register1.len(), register2.len(),
        "Registers must have same size");
    
    let mut gates = Vec::new();
    
    // Hadamard on ancilla
    gates.push(Gate::H(ancilla));
    
    // Controlled-SWAP between registers
    for (&q1, &q2) in register1.iter().zip(register2.iter()) {
        gates.push(Gate::CSWAP(ancilla, q1, q2));
    }
    
    // Hadamard on ancilla
    gates.push(Gate::H(ancilla));
    
    gates
}

/// Hadamard test for computing ⟨ψ|U|ψ⟩
/// 
/// Uses a control qubit to measure real or imaginary part of expectation.
pub fn hadamard_test(
    control: usize,
    _target_qubits: &[usize],
    unitary: &[Gate],
    measure_imaginary: bool,
) -> Vec<Gate> {
    let mut gates = Vec::new();
    
    // Hadamard on control
    gates.push(Gate::H(control));
    
    // S gate if measuring imaginary part
    if measure_imaginary {
        gates.push(Gate::Sdg(control));
    }
    
    // Controlled-U
    // Wrap each gate in controlled version
    for gate in unitary {
        gates.extend(make_controlled(control, gate));
    }
    
    // Final Hadamard
    gates.push(Gate::H(control));
    
    gates
}

fn make_controlled(control: usize, gate: &Gate) -> Vec<Gate> {
    match gate {
        Gate::X(t) => vec![Gate::CX(control, *t)],
        Gate::Y(t) => vec![Gate::CY(control, *t)],
        Gate::Z(t) => vec![Gate::CZ(control, *t)],
        Gate::RZ(t, theta) => vec![Gate::CRZ(control, *t, *theta)],
        Gate::RX(t, theta) => vec![Gate::CRX(control, *t, *theta)],
        Gate::RY(t, theta) => vec![Gate::CRY(control, *t, *theta)],
        // For more complex gates, decompose first
        _ => vec![gate.clone()], // Simplified
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hea_parameter_count() {
        let config = HEAConfig {
            num_qubits: 4,
            num_layers: 2,
            rotation_type: HEALayerType::RyRz,
            entangling_pattern: EntanglingPattern::Linear,
            include_initial_layer: true,
        };
        
        // 4 qubits × 2 params × (1 initial + 2 layers) = 24 parameters
        let params = vec![0.0; 24];
        let circuit = hardware_efficient_ansatz(&config, &params);
        assert!(!circuit.is_empty());
    }

    #[test]
    fn test_qaoa_maxcut() {
        let edges = vec![(0, 1), (1, 2), (2, 0)]; // Triangle
        let gammas = vec![0.5];
        let betas = vec![0.3];
        
        let circuit = qaoa_maxcut(3, &edges, &gammas, &betas);
        
        // Should have 3 H + 3 RZZ + 3 RX
        assert!(circuit.len() >= 9);
    }

    #[test]
    fn test_swap_test() {
        let circuit = swap_test(0, &[1, 2], &[3, 4]);
        
        // H + 2 CSWAP + H
        assert_eq!(circuit.len(), 4);
    }
}
