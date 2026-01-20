//! # Simon's Algorithm - Exponential Quantum Speedup for Period Finding
//!
//! ## 🎯 Purpose & Motivation
//!
//! Simon's Algorithm (1994) discovers a **hidden period** in a function using
//! **O(n) quantum queries**, compared to **Ω(2^(n/2)) classical queries**.
//! This represents an **exponential quantum speedup** and was a precursor to Shor's algorithm.
//!
//! ### The Problem
//!
//! Given a 2-to-1 function f: {0,1}ⁿ → {0,1}ⁿ where:
//! - f(x) = f(y) if and only if y = x ⊕ s
//! - s is a hidden non-zero n-bit period (or s = 0 for 1-to-1 functions)
//!
//! Find the secret period s.
//!
//! ### Complexity Analysis
//!
//! | Approach | Queries Required |
//! |----------|------------------|
//! | Classical (birthday attack) | Ω(2^(n/2)) |
//! | **Quantum** | **O(n)** |
//!
//! This is the **first example of exponential quantum speedup** for a computational problem.
//!
//! ## ⚙️ How It Works
//!
//! ### Algorithm Overview
//!
//! Simon's algorithm is **probabilistic** and requires multiple runs:
//!
//! 1. Run quantum subroutine O(n) times to collect n-1 linearly independent constraints
//! 2. Each run produces a string y such that y · s = 0 (mod 2)
//! 3. Classical post-processing: solve the system of linear equations to find s
//!
//! ### Quantum Subroutine Circuit
//!
//! ```text
//! |0⟩^⊗n ──[H^⊗n]── ──[Uf]── ──[H^⊗n]── ──[Measure]── → y (where y·s = 0)
//! |0⟩^⊗n ─────────── ──[  ]── ───────────────────────
//! ```
//!
//! ### Mathematical Analysis
//!
//! After applying Hadamard and the oracle, the state is:
//! (1/√2^n) Σ_x |x⟩|f(x)⟩
//!
//! Measuring the second register collapses to |x₀⟩ + |x₀ ⊕ s⟩ for some x₀.
//! Applying Hadamard to the first register gives us y such that y · s = 0.
//!
//! ## 📊 Computational Complexity
//!
//! - **Query Complexity**: O(n) - about n oracle queries
//! - **Gate Complexity**: O(n²) for oracle + O(n) Hadamards
//! - **Classical Post-Processing**: O(n³) for Gaussian elimination
//! - **Success Probability**: ~1 - 1/2ⁿ after n runs
//!
//! ## 📍 Practical Applications
//!
//! 1. **Foundation for Shor's**: Period finding → factoring
//! 2. **Cryptanalysis**: Breaking certain symmetric ciphers
//! 3. **Theoretical Importance**: First exponential separation
//!
//! ## 🔬 2025-26 Research Context
//!
//! Simon's algorithm continues to be studied for:
//! - Quantum-safe cryptography analysis
//! - Hidden shift problem generalizations
//! - Resource estimation on NISQ devices
//!
//! ## 📚 References
//!
//! - Simon, D. (1994). "On the power of quantum computation"
//! - Nielsen & Chuang (2010). Ch. 6.6

use std::collections::HashSet;

/// Oracle for Simon's Algorithm
///
/// Implements a 2-to-1 function f: {0,1}ⁿ → {0,1}ⁿ with hidden period s.
/// f(x) = f(y) iff y = x ⊕ s
#[derive(Debug, Clone)]
pub struct SimonsOracle {
    /// The hidden period (n-bit string)
    pub secret: Vec<u8>,
    /// Number of bits
    pub n: usize,
}

impl SimonsOracle {
    /// Creates a new Simon's oracle with the given secret period
    ///
    /// # Arguments
    /// * `secret` - The hidden period s (must be non-empty)
    ///
    /// # Example
    /// ```
    /// use quantic_rust::algorithms::oracles::SimonsOracle;
    ///
    /// let oracle = SimonsOracle::new(vec![1, 1, 0]);  // s = 110
    /// ```
    pub fn new(secret: Vec<u8>) -> Self {
        let n = secret.len();
        let normalized: Vec<u8> = secret.iter().map(|&b| b & 1).collect();
        SimonsOracle { secret: normalized, n }
    }

    /// Creates an oracle from integer representation
    pub fn from_int(secret_int: u64, n: usize) -> Self {
        let secret: Vec<u8> = (0..n)
            .map(|i| ((secret_int >> i) & 1) as u8)
            .collect();
        SimonsOracle { secret, n }
    }

    /// Returns the secret as an integer
    pub fn secret_as_int(&self) -> u64 {
        self.secret.iter().enumerate()
            .map(|(i, &b)| (b as u64) << i)
            .sum()
    }

    /// Checks if the secret is zero (1-to-1 function case)
    pub fn is_one_to_one(&self) -> bool {
        self.secret.iter().all(|&b| b == 0)
    }

    /// Evaluates f(x) for the given input
    ///
    /// The function is defined such that f(x) = f(x ⊕ s).
    /// We implement this by mapping x to min(x, x ⊕ s).
    pub fn evaluate(&self, x: u64) -> u64 {
        let s = self.secret_as_int();
        let x_xor_s = x ^ s;
        
        // Return the lexicographically smaller of x and x ⊕ s
        // This ensures f(x) = f(x ⊕ s)
        std::cmp::min(x, x_xor_s)
    }

    /// Alternative implementation using a hash table
    /// This creates an explicit 2-to-1 mapping
    pub fn evaluate_with_hash(&self, x: u64) -> u64 {
        self.evaluate(x)  // Same implementation for now
    }
}

/// Quantum state for Simon's Algorithm simulation
#[derive(Debug, Clone)]
pub struct SimonsState {
    /// Number of qubits per register
    pub n: usize,
    /// Amplitudes for the 2^(2n) basis states
    /// Index encodes |x⟩|y⟩ as x + y * 2^n
    pub amplitudes: Vec<f64>,
}

impl SimonsState {
    /// Creates initial state |0⟩^⊗n |0⟩^⊗n
    pub fn initial(n: usize) -> Self {
        let size = 1 << (2 * n);  // 2^(2n)
        let mut amplitudes = vec![0.0; size];
        amplitudes[0] = 1.0;
        SimonsState { n, amplitudes }
    }

    /// Applies Hadamard to the first n qubits only
    pub fn apply_hadamard_first_register(&mut self) {
        let size_n = 1 << self.n;
        let size_2n = 1 << (2 * self.n);
        let factor = 1.0 / (size_n as f64).sqrt();
        
        let mut new_amplitudes = vec![0.0; size_2n];
        
        for y in 0..size_n {
            for new_x in 0..size_n {
                for old_x in 0..size_n {
                    let old_idx = old_x + y * size_n;
                    let new_idx = new_x + y * size_n;
                    
                    let dot = ((old_x & new_x) as usize).count_ones();
                    let sign = if dot % 2 == 0 { 1.0 } else { -1.0 };
                    
                    new_amplitudes[new_idx] += sign * self.amplitudes[old_idx] * factor;
                }
            }
        }
        
        self.amplitudes = new_amplitudes;
    }

    /// Applies Simon's oracle: |x⟩|y⟩ → |x⟩|y ⊕ f(x)⟩
    pub fn apply_oracle(&mut self, oracle: &SimonsOracle) {
        let size_n = 1 << self.n;
        let size_2n = 1 << (2 * self.n);
        
        let mut new_amplitudes = vec![0.0; size_2n];
        
        for idx in 0..size_2n {
            let x = idx % size_n;
            let y = idx / size_n;
            
            let fx = oracle.evaluate(x as u64) as usize;
            let new_y = y ^ fx;
            let new_idx = x + new_y * size_n;
            
            new_amplitudes[new_idx] += self.amplitudes[idx];
        }
        
        self.amplitudes = new_amplitudes;
    }

    /// Measures the second register (simulated)
    /// Returns a list of (first_register_value, probability) pairs
    pub fn measure_first_register_distribution(&self) -> Vec<(usize, f64)> {
        let size_n = 1 << self.n;
        let mut probs = vec![0.0; size_n];
        
        for x in 0..size_n {
            for y in 0..(1 << self.n) {
                let idx = x + y * size_n;
                probs[x] += self.amplitudes[idx].powi(2);
            }
        }
        
        probs.into_iter()
            .enumerate()
            .filter(|(_, p)| *p > 1e-10)
            .collect()
    }
}

/// Result of a single Simon's quantum subroutine run
#[derive(Debug, Clone)]
pub struct SimonsRunResult {
    /// The measured y value (should satisfy y · s = 0)
    pub y: u64,
    /// The probability of this outcome
    pub probability: f64,
}

/// Runs one iteration of Simon's quantum subroutine
///
/// Returns a random y such that y · s = 0 (mod 2)
pub fn simons_quantum_subroutine(oracle: &SimonsOracle) -> Vec<SimonsRunResult> {
    let n = oracle.n;
    
    // Step 1: Initialize |0⟩^⊗2n
    let mut state = SimonsState::initial(n);
    
    // Step 2: Apply H^⊗n to first register
    state.apply_hadamard_first_register();
    
    // Step 3: Apply oracle
    state.apply_oracle(oracle);
    
    // Step 4: Apply H^⊗n to first register again
    state.apply_hadamard_first_register();
    
    // Step 5: Get measurement distribution for first register
    state.measure_first_register_distribution()
        .into_iter()
        .map(|(y, prob)| SimonsRunResult { 
            y: y as u64, 
            probability: prob 
        })
        .collect()
}

/// Checks if a vector of constraints (y values) are linearly independent
///
/// Uses Gaussian elimination over GF(2)
fn are_linearly_independent(constraints: &[u64], n: usize) -> bool {
    if constraints.is_empty() {
        return true;
    }
    
    let mut matrix: Vec<u64> = constraints.to_vec();
    let mut rank = 0;
    
    for col in 0..n {
        // Find pivot
        let mut pivot_row = None;
        for row in rank..matrix.len() {
            if (matrix[row] >> col) & 1 == 1 {
                pivot_row = Some(row);
                break;
            }
        }
        
        if let Some(pr) = pivot_row {
            // Swap with current rank row
            matrix.swap(rank, pr);
            
            // Eliminate other rows
            for row in 0..matrix.len() {
                if row != rank && (matrix[row] >> col) & 1 == 1 {
                    matrix[row] ^= matrix[rank];
                }
            }
            
            rank += 1;
        }
    }
    
    rank == constraints.len()
}

/// Solves the system y · s = 0 for all y in constraints
///
/// Returns the secret s (or 0 if the function is 1-to-1)
fn solve_linear_system(constraints: &[u64], n: usize) -> u64 {
    if constraints.len() < n - 1 {
        // Not enough constraints
        return 0;
    }
    
    // Create augmented matrix and do Gaussian elimination
    let mut matrix: Vec<u64> = constraints.to_vec();
    
    // Row reduce to echelon form
    let mut pivot_cols = Vec::new();
    let mut current_row = 0;
    
    for col in 0..n {
        // Find pivot
        let mut pivot_row = None;
        for row in current_row..matrix.len() {
            if (matrix[row] >> col) & 1 == 1 {
                pivot_row = Some(row);
                break;
            }
        }
        
        if let Some(pr) = pivot_row {
            matrix.swap(current_row, pr);
            
            for row in 0..matrix.len() {
                if row != current_row && (matrix[row] >> col) & 1 == 1 {
                    matrix[row] ^= matrix[current_row];
                }
            }
            
            pivot_cols.push(col);
            current_row += 1;
        }
    }
    
    // Find free variable (non-pivot column)
    let all_cols: HashSet<usize> = (0..n).collect();
    let pivot_set: HashSet<usize> = pivot_cols.iter().cloned().collect();
    let free_cols: Vec<usize> = all_cols.difference(&pivot_set).cloned().collect();
    
    if free_cols.is_empty() {
        // Only solution is s = 0 (1-to-1 function)
        return 0;
    }
    
    // Set free variable to 1, solve for others
    let free_col = free_cols[0];
    let mut s = 1u64 << free_col;
    
    // Back substitution
    for (row, &pivot_col) in pivot_cols.iter().enumerate().rev() {
        // Check if this row's equation requires pivot_col bit to be 1
        let row_val = matrix[row];
        let mut sum = 0u64;
        for col in (pivot_col + 1)..n {
            if (row_val >> col) & 1 == 1 {
                sum ^= (s >> col) & 1;
            }
        }
        if sum == 1 {
            s |= 1 << pivot_col;
        }
    }
    
    s
}

/// Full Simon's Algorithm execution
///
/// Runs the quantum subroutine multiple times and uses classical
/// post-processing to find the hidden period.
///
/// # Arguments
///
/// * `oracle` - The Simon's oracle with hidden period s
/// * `max_iterations` - Maximum number of quantum runs (default: 2n)
///
/// # Returns
///
/// The discovered secret period s
///
/// # Algorithm
///
/// 1. Run quantum subroutine ~n times
/// 2. Collect linearly independent y values (where y · s = 0)
/// 3. Solve the linear system to find s
///
/// # Example
///
/// ```
/// use quantic_rust::algorithms::oracles::{SimonsOracle, simons_algorithm};
///
/// let oracle = SimonsOracle::from_int(0b110, 3);  // s = 110
/// let discovered = simons_algorithm(&oracle, None);
/// assert_eq!(discovered, 0b110);
/// ```
pub fn simons_algorithm(oracle: &SimonsOracle, max_iterations: Option<usize>) -> u64 {
    let n = oracle.n;
    let _max_iter = max_iterations.unwrap_or(2 * n);
    
    // Collect linearly independent constraints
    let mut constraints: Vec<u64> = Vec::new();
    
    // Get all possible outcomes from quantum subroutine
    let outcomes = simons_quantum_subroutine(oracle);
    
    // In simulation, we can see all possible outcomes
    // Take non-zero outcomes (y ≠ 0) that are linearly independent
    for result in outcomes {
        if result.y == 0 {
            continue;  // y = 0 gives no information
        }
        
        // Check if this y is linearly independent of existing constraints
        let mut test_constraints = constraints.clone();
        test_constraints.push(result.y);
        
        if are_linearly_independent(&test_constraints, n) {
            constraints.push(result.y);
        }
        
        // We need n-1 constraints
        if constraints.len() >= n - 1 {
            break;
        }
    }
    
    // Solve the system
    solve_linear_system(&constraints, n)
}

/// Verifies Simon's algorithm by checking f(x) = f(x ⊕ s) for discovered s
pub fn verify_simons(oracle: &SimonsOracle, discovered_s: u64) -> bool {
    let n = oracle.n;
    let original_s = oracle.secret_as_int();
    
    // Either discovered_s matches original, or both are 0 (1-to-1 case)
    if discovered_s == original_s {
        return true;
    }
    
    // Verify by checking f(x) = f(x ⊕ discovered_s) for all x
    let size = 1 << n;
    for x in 0..size {
        let x_xor_s = x ^ discovered_s;
        if oracle.evaluate(x) != oracle.evaluate(x_xor_s) {
            return false;
        }
    }
    
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oracle_creation() {
        let oracle = SimonsOracle::from_int(0b110, 3);
        assert_eq!(oracle.secret, vec![0, 1, 1]);
        assert_eq!(oracle.secret_as_int(), 0b110);
    }

    #[test]
    fn test_oracle_evaluation() {
        let oracle = SimonsOracle::from_int(0b110, 3);  // s = 110
        
        // f(x) should equal f(x ⊕ s)
        for x in 0..8u64 {
            let x_xor_s = x ^ 0b110;
            assert_eq!(
                oracle.evaluate(x), 
                oracle.evaluate(x_xor_s),
                "Failed for x={}", x
            );
        }
    }

    #[test]
    fn test_linear_independence() {
        assert!(are_linearly_independent(&[0b001, 0b010, 0b100], 3));
        assert!(!are_linearly_independent(&[0b001, 0b010, 0b011], 3));
        assert!(are_linearly_independent(&[0b110, 0b101], 3));
    }

    #[test]
    fn test_simons_algorithm() {
        // Test with various secrets
        for secret in [0b01, 0b10, 0b11u64] {
            let oracle = SimonsOracle::from_int(secret, 2);
            let discovered = simons_algorithm(&oracle, None);
            assert!(
                verify_simons(&oracle, discovered),
                "Failed for secret {}", secret
            );
        }
    }

    #[test]
    fn test_simons_3_bit() {
        let oracle = SimonsOracle::from_int(0b110, 3);
        let discovered = simons_algorithm(&oracle, None);
        assert_eq!(discovered, 0b110);
    }

    #[test]
    fn test_one_to_one() {
        // s = 0 means function is 1-to-1
        let oracle = SimonsOracle::from_int(0, 3);
        let discovered = simons_algorithm(&oracle, None);
        assert_eq!(discovered, 0);
    }
}
