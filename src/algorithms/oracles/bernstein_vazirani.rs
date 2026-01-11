//! # Bernstein-Vazirani Algorithm - Hidden String Discovery
//!
//! ## 🎯 Purpose & Motivation
//!
//! The Bernstein-Vazirani Algorithm (1993) discovers a **hidden n-bit string** using
//! only **one quantum query**, compared to **n classical queries**. This demonstrates
//! a linear speedup over classical computation.
//!
//! ### The Problem
//!
//! Given a black-box function f(x) = s · x (mod 2), where s is a secret n-bit string
//! and x · s denotes the bitwise inner product, find the secret string s.
//!
//! ### Complexity Analysis
//!
//! | Approach | Queries Required |
//! |----------|------------------|
//! | Classical | n (query with basis vectors e₁, e₂, ..., eₙ) |
//! | **Quantum** | **1** |
//!
//! ## ⚙️ How It Works
//!
//! ### Circuit Diagram
//!
//! ```text
//! |0⟩ ──[H]── ──[Uf]── ──[H]── ──[M]── → s₀
//! |0⟩ ──[H]── ──[   ]── ──[H]── ──[M]── → s₁
//!  ⋮    ⋮          ⋮          ⋮      ⋮     ⋮
//! |0⟩ ──[H]── ──[   ]── ──[H]── ──[M]── → sₙ₋₁
//! |1⟩ ──[H]── ──[   ]── ───────────────
//! ```
//!
//! ### Algorithm Steps
//!
//! 1. **Initialize**: Prepare |0⟩^⊗n |1⟩
//! 2. **Hadamard**: Apply H^⊗(n+1) → uniform superposition
//! 3. **Oracle**: Apply Uf where Uf|x⟩|y⟩ = |x⟩|y ⊕ (s·x)⟩
//! 4. **Hadamard**: Apply H^⊗n to input register → interference reveals s
//! 5. **Measure**: The measurement outcome is exactly s!
//!
//! ### Mathematical Analysis
//!
//! After step 3 (oracle), the input register state is:
//! (1/√2^n) Σ_x (-1)^(s·x) |x⟩
//!
//! The final Hadamard transform acts as a Fourier transform, and:
//! H^⊗n [(1/√2^n) Σ_x (-1)^(s·x) |x⟩] = |s⟩
//!
//! This is because H^⊗n|x⟩ = (1/√2^n) Σ_y (-1)^(x·y) |y⟩
//!
//! ## 📊 Computational Complexity
//!
//! - **Query Complexity**: O(1) - exactly 1 oracle query
//! - **Gate Complexity**: O(n) - 2n+2 Hadamard gates + 1 oracle
//! - **Space Complexity**: O(n) - n+1 qubits
//!
//! ## 📍 Practical Applications
//!
//! 1. **Learning Theory**: Finding hidden linear functions
//! 2. **Cryptanalysis**: Attacks on certain cryptographic schemes
//! 3. **Algorithm Building Block**: Foundation for Simon's algorithm
//! 4. **Quantum Supremacy Demos**: Hardware benchmarking
//!
//! ## 🔬 2025-26 Research Context
//!
//! The 3-qubit Bernstein-Vazirani algorithm has been implemented on:
//! - Single 25-level atomic qudits (arXiv 2025)
//! - Photonic quantum processors
//! - Various NISQ hardware platforms
//!
//! ## 📚 References
//!
//! - Bernstein, E. & Vazirani, U. (1993). "Quantum complexity theory"
//! - Nielsen & Chuang (2010). Ch. 6.5



/// Oracle for Bernstein-Vazirani Algorithm
///
/// Implements the function f(x) = s · x (mod 2) where s is the hidden string.
#[derive(Debug, Clone)]
pub struct BernsteinVaziraniOracle {
    /// The hidden secret string (little-endian bit representation)
    pub secret: Vec<u8>,
}

impl BernsteinVaziraniOracle {
    /// Creates a new oracle with the given secret string
    ///
    /// # Arguments
    /// * `secret` - The hidden n-bit string as a vector of 0s and 1s
    ///
    /// # Example
    /// ```
    /// use quantic_rust::algorithms::oracles::BernsteinVaziraniOracle;
    ///
    /// // Secret string s = 101 (binary) = [1, 0, 1] in little-endian
    /// let oracle = BernsteinVaziraniOracle::new(vec![1, 0, 1]);
    /// ```
    pub fn new(secret: Vec<u8>) -> Self {
        // Normalize bits to 0 or 1
        let normalized: Vec<u8> = secret.iter().map(|&b| b & 1).collect();
        BernsteinVaziraniOracle { secret: normalized }
    }

    /// Creates an oracle from an integer representation
    ///
    /// # Arguments
    /// * `secret_int` - The secret as an unsigned integer
    /// * `n` - Number of bits
    ///
    /// # Example
    /// ```
    /// use quantic_rust::algorithms::oracles::BernsteinVaziraniOracle;
    ///
    /// // Secret = 5 = 101 binary, 3 bits
    /// let oracle = BernsteinVaziraniOracle::from_int(5, 3);
    /// assert_eq!(oracle.secret, vec![1, 0, 1]);
    /// ```
    pub fn from_int(secret_int: u64, n: usize) -> Self {
        let secret: Vec<u8> = (0..n)
            .map(|i| ((secret_int >> i) & 1) as u8)
            .collect();
        BernsteinVaziraniOracle { secret }
    }

    /// Returns the number of bits (n) in the secret
    pub fn num_bits(&self) -> usize {
        self.secret.len()
    }

    /// Evaluates f(x) = s · x mod 2
    ///
    /// # Arguments
    /// * `x` - Input bit vector (little-endian)
    ///
    /// # Returns
    /// The inner product s · x mod 2
    pub fn evaluate(&self, x: &[u8]) -> u8 {
        let mut result = 0u8;
        for (xi, si) in x.iter().zip(self.secret.iter()) {
            result ^= xi & si;
        }
        result
    }

    /// Evaluates f(x) for integer input
    pub fn evaluate_int(&self, x: u64) -> u8 {
        let popcount = (x & self.secret_as_int()).count_ones();
        (popcount & 1) as u8
    }

    /// Returns the secret as an integer
    pub fn secret_as_int(&self) -> u64 {
        self.secret.iter().enumerate()
            .map(|(i, &b)| (b as u64) << i)
            .sum()
    }
}

/// Quantum state for Bernstein-Vazirani simulation
#[derive(Debug, Clone)]
pub struct BernsteinVaziraniState {
    /// Number of input qubits
    pub n: usize,
    /// Amplitude vector (2^n elements)
    pub amplitudes: Vec<f64>,
}

impl BernsteinVaziraniState {
    /// Creates initial state |0⟩^⊗n
    pub fn initial(n: usize) -> Self {
        let size = 1 << n;
        let mut amplitudes = vec![0.0; size];
        amplitudes[0] = 1.0;
        BernsteinVaziraniState { n, amplitudes }
    }

    /// Applies Hadamard to all qubits (Walsh-Hadamard transform)
    pub fn apply_hadamard_all(&mut self) {
        let size = 1 << self.n;
        let factor = 1.0 / (size as f64).sqrt();
        
        let mut new_amplitudes = vec![0.0; size];
        
        for y in 0..size {
            for x in 0..size {
                let dot = (x & y).count_ones();
                let sign = if dot % 2 == 0 { 1.0 } else { -1.0 };
                new_amplitudes[y] += sign * self.amplitudes[x] * factor;
            }
        }
        
        self.amplitudes = new_amplitudes;
    }

    /// Applies the Bernstein-Vazirani oracle (phase kickback)
    ///
    /// Each basis state |x⟩ gets phase (-1)^(s·x)
    pub fn apply_oracle(&mut self, oracle: &BernsteinVaziraniOracle) {
        let size = 1 << self.n;
        
        for x in 0..size {
            if oracle.evaluate_int(x as u64) == 1 {
                self.amplitudes[x] = -self.amplitudes[x];
            }
        }
    }

    /// Returns the index with highest amplitude (should be the secret)
    pub fn measure_most_likely(&self) -> usize {
        self.amplitudes.iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| {
                a.abs().partial_cmp(&b.abs()).unwrap()
            })
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    /// Returns the probability of measuring a specific outcome
    pub fn probability(&self, outcome: usize) -> f64 {
        if outcome < self.amplitudes.len() {
            self.amplitudes[outcome].powi(2)
        } else {
            0.0
        }
    }
}

/// Executes the Bernstein-Vazirani Algorithm
///
/// Finds the hidden string s such that f(x) = s · x (mod 2)
/// using only a single oracle query.
///
/// # Arguments
///
/// * `oracle` - The black-box function encoding the secret
///
/// # Returns
///
/// The discovered secret string as a vector of bits
///
/// # Algorithm
///
/// 1. Initialize |0⟩^⊗n
/// 2. Apply H^⊗n
/// 3. Apply oracle (phase kickback)
/// 4. Apply H^⊗n
/// 5. Measure → outputs exactly s
///
/// # Example
///
/// ```
/// use quantic_rust::algorithms::oracles::{BernsteinVaziraniOracle, bernstein_vazirani_algorithm};
///
/// let oracle = BernsteinVaziraniOracle::new(vec![1, 0, 1, 1]);  // s = 1101
/// let discovered = bernstein_vazirani_algorithm(&oracle);
/// assert_eq!(discovered, vec![1, 0, 1, 1]);
/// ```
///
/// # Complexity
///
/// - **Time**: O(2^n) for classical simulation
/// - **Space**: O(2^n) for state vector
/// - **Oracle Queries**: Exactly 1
pub fn bernstein_vazirani_algorithm(oracle: &BernsteinVaziraniOracle) -> Vec<u8> {
    let n = oracle.num_bits();
    
    // Step 1: Initialize |0⟩^⊗n
    let mut state = BernsteinVaziraniState::initial(n);
    
    // Step 2: Apply H^⊗n
    state.apply_hadamard_all();
    
    // Step 3: Apply oracle
    state.apply_oracle(oracle);
    
    // Step 4: Apply H^⊗n again
    state.apply_hadamard_all();
    
    // Step 5: Measure - in exact simulation, the answer is deterministic
    let result = state.measure_most_likely();
    
    // Convert to bit vector
    (0..n).map(|i| ((result >> i) & 1) as u8).collect()
}

/// Returns the discovered secret as an integer
pub fn bernstein_vazirani_int(oracle: &BernsteinVaziraniOracle) -> u64 {
    let bits = bernstein_vazirani_algorithm(oracle);
    bits.iter().enumerate()
        .map(|(i, &b)| (b as u64) << i)
        .sum()
}

/// Runs Bernstein-Vazirani with execution trace
pub fn bernstein_vazirani_with_trace(
    oracle: &BernsteinVaziraniOracle
) -> (Vec<u8>, Vec<BernsteinVaziraniState>) {
    let n = oracle.num_bits();
    let mut traces = Vec::new();
    
    let mut state = BernsteinVaziraniState::initial(n);
    traces.push(state.clone());
    
    state.apply_hadamard_all();
    traces.push(state.clone());
    
    state.apply_oracle(oracle);
    traces.push(state.clone());
    
    state.apply_hadamard_all();
    traces.push(state.clone());
    
    let result = state.measure_most_likely();
    let bits: Vec<u8> = (0..n).map(|i| ((result >> i) & 1) as u8).collect();
    
    (bits, traces)
}

/// Verifies that the algorithm correctly discovers the secret
///
/// # Arguments
/// * `n` - Number of bits
/// * `secret_int` - The secret to verify against
///
/// # Returns
/// `true` if the algorithm correctly discovers the secret
pub fn verify_bernstein_vazirani(n: usize, secret_int: u64) -> bool {
    let oracle = BernsteinVaziraniOracle::from_int(secret_int, n);
    let discovered = bernstein_vazirani_int(&oracle);
    discovered == secret_int
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oracle_creation() {
        let oracle = BernsteinVaziraniOracle::from_int(5, 3);  // 101
        assert_eq!(oracle.secret, vec![1, 0, 1]);
        assert_eq!(oracle.secret_as_int(), 5);
    }

    #[test]
    fn test_oracle_evaluation() {
        let oracle = BernsteinVaziraniOracle::from_int(5, 3);  // s = 101
        
        // f(x) = s · x mod 2
        assert_eq!(oracle.evaluate_int(0b000), 0);  // 101 · 000 = 0
        assert_eq!(oracle.evaluate_int(0b001), 1);  // 101 · 001 = 1
        assert_eq!(oracle.evaluate_int(0b010), 0);  // 101 · 010 = 0
        assert_eq!(oracle.evaluate_int(0b011), 1);  // 101 · 011 = 1
        assert_eq!(oracle.evaluate_int(0b100), 1);  // 101 · 100 = 1
        assert_eq!(oracle.evaluate_int(0b101), 0);  // 101 · 101 = 0
    }

    #[test]
    fn test_algorithm_basic() {
        // Test with various secrets
        for secret in 0..16u64 {
            let oracle = BernsteinVaziraniOracle::from_int(secret, 4);
            let discovered = bernstein_vazirani_int(&oracle);
            assert_eq!(discovered, secret, "Failed for secret {}", secret);
        }
    }

    #[test]
    fn test_algorithm_vector_form() {
        let oracle = BernsteinVaziraniOracle::new(vec![1, 0, 1, 1]);
        let discovered = bernstein_vazirani_algorithm(&oracle);
        assert_eq!(discovered, vec![1, 0, 1, 1]);
    }

    #[test]
    fn test_larger_secrets() {
        // Test with 8-bit secrets
        for secret in [0, 1, 127, 128, 255u64] {
            let oracle = BernsteinVaziraniOracle::from_int(secret, 8);
            let discovered = bernstein_vazirani_int(&oracle);
            assert_eq!(discovered, secret);
        }
    }

    #[test]
    fn test_verification() {
        assert!(verify_bernstein_vazirani(4, 0b1010));
        assert!(verify_bernstein_vazirani(6, 0b101101));
    }

    #[test]
    fn test_trace_execution() {
        let oracle = BernsteinVaziraniOracle::from_int(5, 3);
        let (result, traces) = bernstein_vazirani_with_trace(&oracle);
        
        assert_eq!(result, vec![1, 0, 1]);  // 5 in little-endian
        assert_eq!(traces.len(), 4);
        
        // Initial state should be |000⟩
        assert!((traces[0].amplitudes[0] - 1.0).abs() < 1e-10);
        
        // Final state should have amplitude 1 at index 5
        assert!((traces[3].amplitudes[5].abs() - 1.0).abs() < 1e-10);
    }
}
