//! # Deutsch-Jozsa Algorithm - Exponential Quantum Speedup
//!
//! ## 🎯 Purpose & Motivation
//!
//! The Deutsch-Jozsa Algorithm (1992) is the **first quantum algorithm to demonstrate
//! exponential speedup** over classical deterministic algorithms. It is a generalization
//! of Deutsch's algorithm to n-bit functions.
//!
//! ### The Problem
//!
//! Given a black-box function f: {0,1}ⁿ → {0,1}, determine if f is:
//! - **Constant**: f(x) is the same for all inputs (always 0 or always 1)
//! - **Balanced**: f(x) = 0 for exactly half the inputs, f(x) = 1 for the other half
//!
//! **Promise**: The function is guaranteed to be either constant or balanced.
//!
//! ### Complexity Analysis
//!
//! | Approach | Queries Required | Best Case | Worst Case |
//! |----------|------------------|-----------|------------|
//! | Classical Deterministic | 2^(n-1) + 1 | 2 | 2^(n-1) + 1 |
//! | Classical Probabilistic | O(1) | 1 | ∞ (w/ certainty) |
//! | **Quantum** | **1** | **1** | **1** |
//!
//! ## ⚙️ How It Works
//!
//! ### Circuit Diagram (n input qubits)
//!
//! ```text
//! |0⟩ ──[H]── ─────────── ──[H]── ──[M]──
//! |0⟩ ──[H]── ─────────── ──[H]── ──[M]──
//!  ⋮    ⋮          ⋮          ⋮      ⋮
//! |0⟩ ──[H]── ──[   Uf   ]── ──[H]── ──[M]──
//! |1⟩ ──[H]── ──[        ]── ──────────────
//! ```
//!
//! ### Algorithm Steps
//!
//! 1. **Initialize**: Prepare |0⟩^⊗n |1⟩ (n qubits in |0⟩, ancilla in |1⟩)
//! 2. **Hadamard**: Apply H to all qubits → creates uniform superposition
//! 3. **Oracle**: Apply Uf → encodes function into phases via phase kickback
//! 4. **Hadamard**: Apply H^⊗n to first n qubits → interference
//! 5. **Measure**: All zeros → Constant, any non-zero → Balanced
//!
//! ### Mathematical Analysis
//!
//! After the algorithm, the state of the first n qubits is:
//!
//! |ψ⟩ = (1/2^n) Σ_x Σ_y (-1)^(f(y) + x·y) |x⟩
//!
//! For a constant function: amplitude of |0...0⟩ = ±1 (constructive interference)
//! For a balanced function: amplitude of |0...0⟩ = 0 (destructive interference)
//!
//! ## 📊 Computational Complexity
//!
//! - **Query Complexity**: O(1) - exactly 1 oracle query
//! - **Gate Complexity**: O(n) - 2n Hadamard gates + 1 oracle
//! - **Space Complexity**: O(n) - n+1 qubits
//!
//! ## 📍 Practical Applications
//!
//! 1. **Theoretical Foundation**: Proves quantum-classical separation
//! 2. **Algorithm Development**: Template for oracle-based algorithms
//! 3. **Educational Tool**: Clear demonstration of quantum parallelism
//! 4. **Benchmarking**: Hardware testing with known expected outcomes
//!
//! ## 🔬 2025-26 Research Context
//!
//! Modern implementations have demonstrated Deutsch-Jozsa on:
//! - 3-qubit systems using 25-level atomic qudits (arXiv 2025)
//! - Photonic quantum processors with high-fidelity gates
//! - Superconducting quantum computers via IBM Quantum
//!
//! ## 📚 References
//!
//! - Deutsch, D. & Jozsa, R. (1992). "Rapid solution of problems by quantum computation"
//! - Cleve et al. (1998). "Quantum algorithms revisited"
//! - Nielsen & Chuang (2010). Ch. 1.4.4



/// Represents different types of n-bit oracles for Deutsch-Jozsa
#[derive(Debug, Clone)]
pub enum DeutschJozsaOracle {
    /// Constant function: f(x) = 0 for all x
    Constant0,
    /// Constant function: f(x) = 1 for all x
    Constant1,
    /// Balanced function defined by a secret string s: f(x) = x · s (mod 2)
    /// This creates a balanced function for any non-zero s
    BalancedInnerProduct { secret: Vec<u8> },
    /// Balanced function: f(x) = parity of x (XOR of all bits)
    /// This is balanced for even n
    BalancedParity,
    /// Custom balanced function defined by a lookup table
    /// The table maps input indices to output bits
    BalancedCustom { table: Vec<u8> },
}

impl DeutschJozsaOracle {
    /// Evaluates the oracle function at a given input
    ///
    /// # Arguments
    /// * `x` - Input as a binary vector (little-endian)
    ///
    /// # Returns
    /// The function output f(x) ∈ {0, 1}
    pub fn evaluate(&self, x: &[u8]) -> u8 {
        match self {
            DeutschJozsaOracle::Constant0 => 0,
            DeutschJozsaOracle::Constant1 => 1,
            DeutschJozsaOracle::BalancedInnerProduct { secret } => {
                // Compute inner product x · s mod 2
                let mut result = 0u8;
                for (xi, si) in x.iter().zip(secret.iter()) {
                    result ^= xi & si;
                }
                result
            }
            DeutschJozsaOracle::BalancedParity => {
                // XOR of all bits
                x.iter().fold(0u8, |acc, &bit| acc ^ bit)
            }
            DeutschJozsaOracle::BalancedCustom { table } => {
                // Convert x to index
                let index: usize = x.iter().enumerate()
                    .map(|(i, &bit)| (bit as usize) << i)
                    .sum();
                table.get(index).copied().unwrap_or(0)
            }
        }
    }

    /// Returns whether this oracle is constant
    pub fn is_constant(&self) -> bool {
        matches!(self, DeutschJozsaOracle::Constant0 | DeutschJozsaOracle::Constant1)
    }

    /// Returns the number of bits (n) for this oracle
    pub fn num_bits(&self) -> Option<usize> {
        match self {
            DeutschJozsaOracle::BalancedInnerProduct { secret } => Some(secret.len()),
            DeutschJozsaOracle::BalancedCustom { table } => {
                // n bits means 2^n entries
                Some((table.len() as f64).log2() as usize)
            }
            _ => None, // Constant functions work for any n
        }
    }
}

/// Result of running Deutsch-Jozsa Algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeutschJozsaResult {
    /// The function is constant
    Constant,
    /// The function is balanced
    Balanced,
}

/// Quantum state vector for Deutsch-Jozsa simulation
///
/// Stores complex amplitudes for all 2^(n+1) basis states.
/// For efficiency, we only track the n input qubits' state after
/// tracing out the ancilla qubit (which is in |-⟩ throughout).
#[derive(Debug, Clone)]
pub struct DeutschJozsaState {
    /// Number of input qubits
    pub n: usize,
    /// Amplitude vector (2^n elements, indexed by basis state)
    /// amplitudes[i] corresponds to state |i⟩ in little-endian binary
    pub amplitudes: Vec<f64>,
}

impl DeutschJozsaState {
    /// Creates the initial state |0⟩^⊗n
    ///
    /// We don't explicitly track the ancilla qubit since it stays in |-⟩
    pub fn initial(n: usize) -> Self {
        let size = 1 << n;  // 2^n
        let mut amplitudes = vec![0.0; size];
        amplitudes[0] = 1.0;  // |00...0⟩
        DeutschJozsaState { n, amplitudes }
    }

    /// Applies Hadamard gate to all n qubits
    ///
    /// H^⊗n transforms |0⟩^⊗n into uniform superposition (1/√2^n) Σ|x⟩
    pub fn apply_hadamard_all(&mut self) {
        let size = 1 << self.n;
        let factor = 1.0 / (size as f64).sqrt();
        
        // Hadamard transform is equivalent to Walsh-Hadamard transform
        // We implement it using the fast Walsh-Hadamard transform (FWHT)
        let mut new_amplitudes = vec![0.0; size];
        
        for x in 0..size {
            for y in 0..size {
                // (-1)^(x·y) where x·y is bitwise AND popcount
                let dot_product = (x & y).count_ones();
                let sign = if dot_product % 2 == 0 { 1.0 } else { -1.0 };
                new_amplitudes[x] += sign * self.amplitudes[y] * factor;
            }
        }
        
        self.amplitudes = new_amplitudes;
    }

    /// Applies the Deutsch-Jozsa oracle with phase kickback
    ///
    /// The oracle Uf acts as: |x⟩|-⟩ → (-1)^f(x) |x⟩|-⟩
    /// Since ancilla is in |-⟩ = (|0⟩ - |1⟩)/√2, we get phase kickback
    pub fn apply_oracle(&mut self, oracle: &DeutschJozsaOracle) {
        let size = 1 << self.n;
        
        for x in 0..size {
            // Convert index to bit vector
            let bits: Vec<u8> = (0..self.n)
                .map(|i| ((x >> i) & 1) as u8)
                .collect();
            
            // If f(x) = 1, flip the phase
            if oracle.evaluate(&bits) == 1 {
                self.amplitudes[x] = -self.amplitudes[x];
            }
        }
    }

    /// Gets the probability of measuring all zeros
    ///
    /// For Deutsch-Jozsa:
    /// - Constant function: P(|0...0⟩) = 1
    /// - Balanced function: P(|0...0⟩) = 0
    pub fn prob_all_zeros(&self) -> f64 {
        self.amplitudes[0].powi(2)
    }

    /// Measures the state and returns the result
    pub fn measure(&self) -> DeutschJozsaResult {
        // With exact simulation, we check if |0...0⟩ has non-zero amplitude
        if self.amplitudes[0].abs() > 0.5 {
            DeutschJozsaResult::Constant
        } else {
            DeutschJozsaResult::Balanced
        }
    }
}

/// Executes the Deutsch-Jozsa Algorithm
///
/// Determines whether a promised constant-or-balanced function is constant
/// or balanced using a single quantum query.
///
/// # Arguments
///
/// * `oracle` - The black-box function to analyze
/// * `n` - Number of input bits
///
/// # Returns
///
/// `DeutschJozsaResult::Constant` if f is constant, `DeutschJozsaResult::Balanced` otherwise
///
/// # Algorithm
///
/// 1. Initialize |0⟩^⊗n
/// 2. Apply H^⊗n → uniform superposition
/// 3. Apply Uf → phase kickback encodes f(x) into phases
/// 4. Apply H^⊗n → interference
/// 5. Measure → all zeros means constant
///
/// # Example
///
/// ```
/// use quantic_rust::algorithms::oracles::{DeutschJozsaOracle, deutsch_jozsa_algorithm, DeutschJozsaResult};
///
/// // Test constant function
/// let result = deutsch_jozsa_algorithm(&DeutschJozsaOracle::Constant0, 3);
/// assert_eq!(result, DeutschJozsaResult::Constant);
///
/// // Test balanced function (inner product with secret = [1,1,0])
/// let balanced = DeutschJozsaOracle::BalancedInnerProduct { 
///     secret: vec![1, 1, 0] 
/// };
/// let result = deutsch_jozsa_algorithm(&balanced, 3);
/// assert_eq!(result, DeutschJozsaResult::Balanced);
/// ```
///
/// # Complexity
///
/// - **Time**: O(2^n) for simulation (quantum hardware would be O(n))
/// - **Space**: O(2^n) for state vector (n+1 qubits on hardware)
/// - **Oracle Queries**: Exactly 1
pub fn deutsch_jozsa_algorithm(oracle: &DeutschJozsaOracle, n: usize) -> DeutschJozsaResult {
    // Step 1: Initialize |0⟩^⊗n
    let mut state = DeutschJozsaState::initial(n);
    
    // Step 2: Apply H^⊗n to create superposition (1/√2^n)Σ|x⟩
    state.apply_hadamard_all();
    
    // Step 3: Apply oracle Uf with phase kickback
    // After this: (1/√2^n) Σ (-1)^f(x) |x⟩
    state.apply_oracle(oracle);
    
    // Step 4: Apply H^⊗n again (interference)
    // For constant f: amplitude of |0...0⟩ = ±1
    // For balanced f: amplitude of |0...0⟩ = 0
    state.apply_hadamard_all();
    
    // Step 5: Measure
    state.measure()
}

/// Runs Deutsch-Jozsa with execution trace for visualization
///
/// Returns intermediate states at each step of the algorithm.
pub fn deutsch_jozsa_with_trace(
    oracle: &DeutschJozsaOracle, 
    n: usize
) -> (DeutschJozsaState, DeutschJozsaResult, Vec<DeutschJozsaState>) {
    let mut traces = Vec::new();
    
    let mut state = DeutschJozsaState::initial(n);
    traces.push(state.clone());
    
    state.apply_hadamard_all();
    traces.push(state.clone());
    
    state.apply_oracle(oracle);
    traces.push(state.clone());
    
    state.apply_hadamard_all();
    traces.push(state.clone());
    
    let result = state.measure();
    (state, result, traces)
}

/// Creates a valid balanced oracle using inner product with a random non-zero secret
///
/// # Arguments
/// * `n` - Number of input bits
/// * `seed` - Seed for deterministic secret generation
///
/// # Returns
/// A balanced oracle guaranteed to output 0 for exactly half the inputs
pub fn create_balanced_oracle(n: usize, seed: u64) -> DeutschJozsaOracle {
    // Simple deterministic "random" non-zero secret based on seed
    let mut secret = Vec::with_capacity(n);
    let mut s = seed;
    let mut has_one = false;
    
    for _i in 0..n {
        s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
        let bit = ((s >> 33) & 1) as u8;
        secret.push(bit);
        if bit == 1 {
            has_one = true;
        }
    }
    
    // Ensure at least one bit is 1 (otherwise it's constant)
    if !has_one {
        secret[0] = 1;
    }
    
    DeutschJozsaOracle::BalancedInnerProduct { secret }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_oracles() {
        for n in 1..=5 {
            assert_eq!(
                deutsch_jozsa_algorithm(&DeutschJozsaOracle::Constant0, n),
                DeutschJozsaResult::Constant,
                "Constant0 should be constant for n={}", n
            );
            assert_eq!(
                deutsch_jozsa_algorithm(&DeutschJozsaOracle::Constant1, n),
                DeutschJozsaResult::Constant,
                "Constant1 should be constant for n={}", n
            );
        }
    }

    #[test]
    fn test_balanced_inner_product() {
        // Inner product with non-zero secret is always balanced
        let oracle = DeutschJozsaOracle::BalancedInnerProduct { 
            secret: vec![1, 0, 1] 
        };
        assert_eq!(
            deutsch_jozsa_algorithm(&oracle, 3),
            DeutschJozsaResult::Balanced
        );
    }

    #[test]
    fn test_balanced_parity() {
        // Parity is balanced for any n ≥ 1
        for n in 1..=5 {
            assert_eq!(
                deutsch_jozsa_algorithm(&DeutschJozsaOracle::BalancedParity, n),
                DeutschJozsaResult::Balanced,
                "Parity should be balanced for n={}", n
            );
        }
    }

    #[test]
    fn test_oracle_evaluation() {
        let ip_oracle = DeutschJozsaOracle::BalancedInnerProduct { 
            secret: vec![1, 1, 0] 
        };
        
        // x·s where s = [1,1,0]
        assert_eq!(ip_oracle.evaluate(&[0, 0, 0]), 0);  // 0
        assert_eq!(ip_oracle.evaluate(&[1, 0, 0]), 1);  // 1
        assert_eq!(ip_oracle.evaluate(&[0, 1, 0]), 1);  // 1
        assert_eq!(ip_oracle.evaluate(&[1, 1, 0]), 0);  // 0 (1+1 mod 2)
    }

    #[test]
    fn test_create_balanced_oracle() {
        for seed in 0..10u64 {
            let oracle = create_balanced_oracle(5, seed);
            assert_eq!(
                deutsch_jozsa_algorithm(&oracle, 5),
                DeutschJozsaResult::Balanced
            );
        }
    }

    #[test]
    fn test_trace_execution() {
        let (_, result, traces) = deutsch_jozsa_with_trace(
            &DeutschJozsaOracle::Constant0, 
            2
        );
        
        assert_eq!(result, DeutschJozsaResult::Constant);
        assert_eq!(traces.len(), 4);
        
        // Initial state should be |00⟩
        assert!((traces[0].amplitudes[0] - 1.0).abs() < 1e-10);
        
        // After first Hadamard, should be uniform superposition
        let expected_amp = 1.0 / 2.0;  // 1/√4 = 0.5
        for amp in &traces[1].amplitudes {
            assert!((amp.abs() - expected_amp).abs() < 1e-10);
        }
    }
}
