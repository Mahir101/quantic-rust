//! # Deutsch's Algorithm - The First Quantum Algorithm
//!
//! ## 🎯 Purpose & Motivation
//!
//! Deutsch's Algorithm (1985) is historically the **first quantum algorithm** ever devised,
//! created by David Deutsch. It demonstrates that a quantum computer can determine a global
//! property of a function with **fewer queries** than any classical algorithm.
//!
//! ### The Problem
//!
//! Given a black-box function f: {0,1} → {0,1}, determine if f is:
//! - **Constant**: f(0) = f(1) (both outputs are the same)
//! - **Balanced**: f(0) ≠ f(1) (outputs are different)
//!
//! ### Classical vs Quantum Complexity
//!
//! | Approach | Queries Required |
//! |----------|------------------|
//! | Classical | 2 (must check both inputs) |
//! | Quantum | 1 (single oracle query) |
//!
//! ## ⚙️ How It Works
//!
//! ### Circuit Diagram
//!
//! ```text
//! |0⟩ ──[H]── ──[Uf]── ──[H]── ──[M]──
//!                │
//! |1⟩ ──[H]── ──[  ]── ────────────────
//! ```
//!
//! ### Algorithm Steps
//!
//! 1. **Initialize**: Prepare |0⟩|1⟩ state
//! 2. **Hadamard**: Apply H to both qubits → creates superposition
//! 3. **Oracle**: Apply Uf, which performs |x⟩|y⟩ → |x⟩|y ⊕ f(x)⟩
//! 4. **Hadamard**: Apply H to first qubit (interference)
//! 5. **Measure**: First qubit reveals function type:
//!    - |0⟩ → Constant
//!    - |1⟩ → Balanced
//!
//! ### Mathematical Analysis
//!
//! After the oracle, the first qubit is in state:
//! - Constant (f(0)=f(1)): ±|0⟩ → measurement gives 0
//! - Balanced (f(0)≠f(1)): ±|1⟩ → measurement gives 1
//!
//! The phase kickback trick encodes f(0)⊕f(1) into the first qubit's phase,
//! which the final Hadamard converts into a measurable difference.
//!
//! ## 📊 Computational Complexity
//!
//! - **Query Complexity**: O(1) - exactly 1 oracle query
//! - **Gate Complexity**: O(1) - 4 Hadamard gates + 1 oracle
//! - **Space Complexity**: O(1) - exactly 2 qubits
//!
//! ## 📍 Practical Applications
//!
//! While Deutsch's algorithm solves a contrived problem, its importance lies in:
//!
//! 1. **Proof of Concept**: First demonstration of quantum speedup
//! 2. **Educational Tool**: Introduces quantum parallelism and interference
//! 3. **Foundation**: Precursor to Deutsch-Jozsa, which generalizes to n bits
//! 4. **Hardware Testing**: Simple benchmark for quantum computers
//!
//! ## 🔬 2025-26 Research Context
//!
//! Modern implementations on qudit-based architectures (2025) have demonstrated
//! this algorithm on single 25-level atomic qudits, achieving high-fidelity
//! execution as reported in arXiv 2025 papers on qudit quantum computing.
//!
//! ## 📚 References
//!
//! - Deutsch, D. (1985). Proc. R. Soc. Lond. A 400, 97-117
//! - Nielsen & Chuang (2010). "Quantum Computation and Quantum Information", Ch. 1.4.3

use std::f64::consts::FRAC_1_SQRT_2;

/// Represents the four possible single-bit boolean functions f: {0,1} → {0,1}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeutschOracle {
    /// f(x) = 0 for all x (constant)
    Constant0,
    /// f(x) = 1 for all x (constant)
    Constant1,
    /// f(x) = x (balanced, identity function)
    BalancedIdentity,
    /// f(x) = NOT x (balanced, negation function)
    BalancedNegation,
}

impl DeutschOracle {
    /// Evaluates the oracle function at a given input
    ///
    /// # Arguments
    /// * `x` - Input bit (0 or 1)
    ///
    /// # Returns
    /// The function output f(x)
    ///
    /// # Example
    /// ```
    /// use quantic_rust::algorithms::oracles::DeutschOracle;
    ///
    /// let oracle = DeutschOracle::BalancedIdentity;
    /// assert_eq!(oracle.evaluate(0), 0);
    /// assert_eq!(oracle.evaluate(1), 1);
    /// ```
    pub fn evaluate(&self, x: u8) -> u8 {
        match self {
            DeutschOracle::Constant0 => 0,
            DeutschOracle::Constant1 => 1,
            DeutschOracle::BalancedIdentity => x & 1,
            DeutschOracle::BalancedNegation => 1 - (x & 1),
        }
    }

    /// Returns whether this oracle represents a constant function
    pub fn is_constant(&self) -> bool {
        matches!(self, DeutschOracle::Constant0 | DeutschOracle::Constant1)
    }

    /// Returns whether this oracle represents a balanced function
    pub fn is_balanced(&self) -> bool {
        !self.is_constant()
    }
}

/// Result of running Deutsch's Algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeutschResult {
    /// The function is constant (f(0) = f(1))
    Constant,
    /// The function is balanced (f(0) ≠ f(1))
    Balanced,
}

/// Quantum state representation for Deutsch's Algorithm
///
/// The state vector represents the 2-qubit system in the computational basis:
/// |00⟩, |01⟩, |10⟩, |11⟩
///
/// We use complex amplitudes for generality, though Deutsch's algorithm
/// only requires real amplitudes.
#[derive(Debug, Clone)]
pub struct DeutschState {
    /// Amplitude of |00⟩
    pub amp_00: f64,
    /// Amplitude of |01⟩
    pub amp_01: f64,
    /// Amplitude of |10⟩
    pub amp_10: f64,
    /// Amplitude of |11⟩
    pub amp_11: f64,
}

impl DeutschState {
    /// Creates the initial state |01⟩
    ///
    /// We start with the first qubit as |0⟩ and the second as |1⟩,
    /// which is required for the phase kickback trick.
    pub fn initial() -> Self {
        DeutschState {
            amp_00: 0.0,
            amp_01: 1.0,  // |01⟩ state
            amp_10: 0.0,
            amp_11: 0.0,
        }
    }

    /// Applies Hadamard gate to both qubits
    ///
    /// The Hadamard gate transforms:
    /// - |0⟩ → (|0⟩ + |1⟩) / √2
    /// - |1⟩ → (|0⟩ - |1⟩) / √2
    ///
    /// For two qubits, this creates the state:
    /// |01⟩ → (|0⟩ + |1⟩)(|0⟩ - |1⟩) / 2 = (|00⟩ - |01⟩ + |10⟩ - |11⟩) / 2
    pub fn apply_hadamard_all(&mut self) {
        // H ⊗ H transformation
        let new_00 = FRAC_1_SQRT_2 * FRAC_1_SQRT_2 * 
            (self.amp_00 + self.amp_01 + self.amp_10 + self.amp_11);
        let new_01 = FRAC_1_SQRT_2 * FRAC_1_SQRT_2 * 
            (self.amp_00 - self.amp_01 + self.amp_10 - self.amp_11);
        let new_10 = FRAC_1_SQRT_2 * FRAC_1_SQRT_2 * 
            (self.amp_00 + self.amp_01 - self.amp_10 - self.amp_11);
        let new_11 = FRAC_1_SQRT_2 * FRAC_1_SQRT_2 * 
            (self.amp_00 - self.amp_01 - self.amp_10 + self.amp_11);

        self.amp_00 = new_00;
        self.amp_01 = new_01;
        self.amp_10 = new_10;
        self.amp_11 = new_11;
    }

    /// Applies Hadamard gate to the first qubit only
    ///
    /// Used in the final step to convert phase information to amplitude.
    pub fn apply_hadamard_first(&mut self) {
        let new_00 = FRAC_1_SQRT_2 * (self.amp_00 + self.amp_10);
        let new_01 = FRAC_1_SQRT_2 * (self.amp_01 + self.amp_11);
        let new_10 = FRAC_1_SQRT_2 * (self.amp_00 - self.amp_10);
        let new_11 = FRAC_1_SQRT_2 * (self.amp_01 - self.amp_11);

        self.amp_00 = new_00;
        self.amp_01 = new_01;
        self.amp_10 = new_10;
        self.amp_11 = new_11;
    }

    /// Applies the Deutsch oracle Uf
    ///
    /// The oracle performs the transformation:
    /// |x⟩|y⟩ → |x⟩|y ⊕ f(x)⟩
    ///
    /// This is the key quantum operation that queries the function
    /// in superposition.
    pub fn apply_oracle(&mut self, oracle: &DeutschOracle) {
        // For each basis state |xy⟩, we compute y ⊕ f(x)
        // If f(x) = 1, we swap the amplitudes of |x0⟩ and |x1⟩
        
        // For x = 0: check if f(0) = 1
        if oracle.evaluate(0) == 1 {
            std::mem::swap(&mut self.amp_00, &mut self.amp_01);
        }
        
        // For x = 1: check if f(1) = 1
        if oracle.evaluate(1) == 1 {
            std::mem::swap(&mut self.amp_10, &mut self.amp_11);
        }
    }

    /// Measures the first qubit in the computational basis
    ///
    /// Returns the probability of measuring |0⟩ on the first qubit.
    /// In the ideal case:
    /// - P(0) ≈ 1 for constant functions
    /// - P(0) ≈ 0 for balanced functions
    pub fn measure_first_qubit_prob_zero(&self) -> f64 {
        self.amp_00.powi(2) + self.amp_01.powi(2)
    }

    /// Performs a deterministic measurement based on amplitudes
    ///
    /// Returns `DeutschResult::Constant` if P(|0⟩) > 0.5, else `Balanced`
    pub fn measure(&self) -> DeutschResult {
        if self.measure_first_qubit_prob_zero() > 0.5 {
            DeutschResult::Constant
        } else {
            DeutschResult::Balanced
        }
    }
}

/// Executes Deutsch's Algorithm to determine if an oracle function is constant or balanced
///
/// This function simulates the complete quantum circuit for Deutsch's Algorithm,
/// demonstrating the quantum advantage of determining a global function property
/// with a single oracle query.
///
/// # Arguments
///
/// * `oracle` - The black-box function to analyze
///
/// # Returns
///
/// `DeutschResult::Constant` if f(0) = f(1), `DeutschResult::Balanced` otherwise
///
/// # Algorithm Steps
///
/// 1. Initialize state |01⟩
/// 2. Apply H ⊗ H to create superposition
/// 3. Apply oracle Uf (phase kickback occurs)
/// 4. Apply H ⊗ I to first qubit
/// 5. Measure first qubit
///
/// # Example
///
/// ```
/// use quantic_rust::algorithms::oracles::{DeutschOracle, deutsch_algorithm, DeutschResult};
///
/// // Test a constant function
/// let result = deutsch_algorithm(&DeutschOracle::Constant0);
/// assert_eq!(result, DeutschResult::Constant);
///
/// // Test a balanced function
/// let result = deutsch_algorithm(&DeutschOracle::BalancedIdentity);
/// assert_eq!(result, DeutschResult::Balanced);
/// ```
///
/// # Complexity
///
/// - **Time**: O(1) - constant time simulation
/// - **Space**: O(1) - only 4 amplitudes stored
/// - **Oracle Queries**: Exactly 1
pub fn deutsch_algorithm(oracle: &DeutschOracle) -> DeutschResult {
    // Step 1: Initialize |01⟩
    let mut state = DeutschState::initial();
    
    // Step 2: Apply H to both qubits
    // Creates: (|0⟩ + |1⟩)(|0⟩ - |1⟩) / 2
    state.apply_hadamard_all();
    
    // Step 3: Apply the oracle Uf
    // This is where quantum parallelism happens - we query f(0) AND f(1) simultaneously
    // Phase kickback encodes f(0) ⊕ f(1) into the first qubit's relative phase
    state.apply_oracle(oracle);
    
    // Step 4: Apply H to first qubit only
    // Converts phase difference into amplitude difference (interference)
    state.apply_hadamard_first();
    
    // Step 5: Measure the first qubit
    state.measure()
}

/// Extended execution that returns the full quantum state for debugging/visualization
///
/// This is useful for educational purposes to see how the state evolves
/// through each step of the algorithm.
///
/// # Returns
///
/// A tuple containing:
/// - The final quantum state
/// - The measurement result
/// - A vector of intermediate states for visualization
pub fn deutsch_algorithm_with_trace(
    oracle: &DeutschOracle
) -> (DeutschState, DeutschResult, Vec<DeutschState>) {
    let mut traces = Vec::new();
    
    // Step 1: Initial state |01⟩
    let mut state = DeutschState::initial();
    traces.push(state.clone());
    
    // Step 2: After H ⊗ H
    state.apply_hadamard_all();
    traces.push(state.clone());
    
    // Step 3: After oracle
    state.apply_oracle(oracle);
    traces.push(state.clone());
    
    // Step 4: After H on first qubit
    state.apply_hadamard_first();
    traces.push(state.clone());
    
    // Step 5: Measurement
    let result = state.measure();
    
    (state, result, traces)
}

/// Circuit representation for Deutsch's Algorithm
///
/// This structure provides a gate-level description suitable for
/// transpilation to actual quantum hardware or other simulators.
#[derive(Debug, Clone)]
pub struct DeutschCircuit {
    /// Number of qubits (always 2 for Deutsch's algorithm)
    pub num_qubits: usize,
    /// Gate sequence as (gate_type, target_qubits)
    pub gates: Vec<DeutschGate>,
}

/// Gates used in Deutsch's Algorithm
#[derive(Debug, Clone, Copy)]
pub enum DeutschGate {
    /// Hadamard gate on qubit index
    Hadamard(usize),
    /// Pauli-X gate on qubit index (for initialization)
    PauliX(usize),
    /// CNOT gate (control, target)
    CNOT(usize, usize),
    /// Oracle gate (represented abstractly)
    Oracle(DeutschOracle),
}

impl DeutschCircuit {
    /// Creates the standard Deutsch's Algorithm circuit
    ///
    /// # Arguments
    ///
    /// * `oracle` - The oracle function to embed in the circuit
    ///
    /// # Returns
    ///
    /// A `DeutschCircuit` ready for simulation or transpilation
    pub fn new(oracle: DeutschOracle) -> Self {
        DeutschCircuit {
            num_qubits: 2,
            gates: vec![
                // Prepare |01⟩ from |00⟩
                DeutschGate::PauliX(1),
                // Apply H to both qubits
                DeutschGate::Hadamard(0),
                DeutschGate::Hadamard(1),
                // Apply oracle
                DeutschGate::Oracle(oracle),
                // Apply H to first qubit for interference
                DeutschGate::Hadamard(0),
            ],
        }
    }

    /// Returns the circuit depth (number of sequential gate layers)
    pub fn depth(&self) -> usize {
        // Simplified: gates that can be parallelized count as 1 layer
        // H(0), H(1) can be parallel, oracle is 1, final H(0) is 1
        4
    }

    /// Returns the total gate count
    pub fn gate_count(&self) -> usize {
        self.gates.len()
    }

    /// Converts to QASM-like string representation
    pub fn to_qasm(&self) -> String {
        let mut qasm = String::from("OPENQASM 2.0;\ninclude \"qelib1.inc\";\nqreg q[2];\ncreg c[1];\n\n");
        
        for gate in &self.gates {
            match gate {
                DeutschGate::Hadamard(i) => {
                    qasm.push_str(&format!("h q[{}];\n", i));
                }
                DeutschGate::PauliX(i) => {
                    qasm.push_str(&format!("x q[{}];\n", i));
                }
                DeutschGate::CNOT(c, t) => {
                    qasm.push_str(&format!("cx q[{}], q[{}];\n", c, t));
                }
                DeutschGate::Oracle(o) => {
                    // Oracle implementation depends on the function
                    match o {
                        DeutschOracle::Constant0 => {
                            qasm.push_str("// Oracle: f(x) = 0 (identity)\n");
                        }
                        DeutschOracle::Constant1 => {
                            qasm.push_str("// Oracle: f(x) = 1\n");
                            qasm.push_str("x q[1];\n");
                        }
                        DeutschOracle::BalancedIdentity => {
                            qasm.push_str("// Oracle: f(x) = x\n");
                            qasm.push_str("cx q[0], q[1];\n");
                        }
                        DeutschOracle::BalancedNegation => {
                            qasm.push_str("// Oracle: f(x) = NOT x\n");
                            qasm.push_str("x q[0];\n");
                            qasm.push_str("cx q[0], q[1];\n");
                            qasm.push_str("x q[0];\n");
                        }
                    }
                }
            }
        }
        
        qasm.push_str("\nmeasure q[0] -> c[0];\n");
        qasm
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_oracles() {
        // Both constant oracles should return Constant
        assert_eq!(
            deutsch_algorithm(&DeutschOracle::Constant0),
            DeutschResult::Constant
        );
        assert_eq!(
            deutsch_algorithm(&DeutschOracle::Constant1),
            DeutschResult::Constant
        );
    }

    #[test]
    fn test_balanced_oracles() {
        // Both balanced oracles should return Balanced
        assert_eq!(
            deutsch_algorithm(&DeutschOracle::BalancedIdentity),
            DeutschResult::Balanced
        );
        assert_eq!(
            deutsch_algorithm(&DeutschOracle::BalancedNegation),
            DeutschResult::Balanced
        );
    }

    #[test]
    fn test_oracle_evaluation() {
        assert_eq!(DeutschOracle::Constant0.evaluate(0), 0);
        assert_eq!(DeutschOracle::Constant0.evaluate(1), 0);
        assert_eq!(DeutschOracle::Constant1.evaluate(0), 1);
        assert_eq!(DeutschOracle::Constant1.evaluate(1), 1);
        assert_eq!(DeutschOracle::BalancedIdentity.evaluate(0), 0);
        assert_eq!(DeutschOracle::BalancedIdentity.evaluate(1), 1);
        assert_eq!(DeutschOracle::BalancedNegation.evaluate(0), 1);
        assert_eq!(DeutschOracle::BalancedNegation.evaluate(1), 0);
    }

    #[test]
    fn test_is_constant_balanced() {
        assert!(DeutschOracle::Constant0.is_constant());
        assert!(DeutschOracle::Constant1.is_constant());
        assert!(!DeutschOracle::BalancedIdentity.is_constant());
        assert!(!DeutschOracle::BalancedNegation.is_constant());
        
        assert!(!DeutschOracle::Constant0.is_balanced());
        assert!(!DeutschOracle::Constant1.is_balanced());
        assert!(DeutschOracle::BalancedIdentity.is_balanced());
        assert!(DeutschOracle::BalancedNegation.is_balanced());
    }

    #[test]
    fn test_circuit_generation() {
        let circuit = DeutschCircuit::new(DeutschOracle::BalancedIdentity);
        assert_eq!(circuit.num_qubits, 2);
        assert_eq!(circuit.gate_count(), 5);
        
        let qasm = circuit.to_qasm();
        assert!(qasm.contains("h q[0]"));
        assert!(qasm.contains("cx q[0], q[1]"));
    }

    #[test]
    fn test_trace_execution() {
        let (final_state, result, traces) = 
            deutsch_algorithm_with_trace(&DeutschOracle::BalancedIdentity);
        
        assert_eq!(result, DeutschResult::Balanced);
        assert_eq!(traces.len(), 4);  // Initial, after H, after oracle, after final H
        
        // Initial state should be |01⟩
        assert!((traces[0].amp_01 - 1.0).abs() < 1e-10);
    }
}
