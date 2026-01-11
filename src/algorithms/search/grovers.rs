//! # Grover's Search Algorithm - Quantum Database Search
//!
//! ## 🎯 Purpose & Motivation
//!
//! Grover's Algorithm (1996) provides a **quadratic speedup** for unstructured search
//! problems. Given a database of N items with M solutions, Grover's algorithm finds
//! a solution in **O(√(N/M))** queries, compared to **O(N/M)** classically.
//!
//! This is one of the most important quantum algorithms because:
//! 1. It's **provably optimal** for unstructured search
//! 2. It has **wide applicability** (SAT solving, optimization, machine learning)
//! 3. It demonstrates **quantum amplitude amplification**
//!
//! ## ⚙️ How It Works
//!
//! ### The Problem
//!
//! Given a black-box function f: {0,1}ⁿ → {0,1}, find x such that f(x) = 1.
//!
//! ### Circuit Diagram
//!
//! ```text
//! |0⟩^⊗n ──[H^⊗n]── ─┌──────────────────┐── ──[M]──
//!                     │  Repeat O(√N)   │
//!                     │  ┌──────────┐   │
//!                     │  │  Oracle  │   │
//!                     │  │   Oₓ     │   │
//!                     │  └──────────┘   │
//!                     │  ┌──────────┐   │
//!                     │  │ Diffuser │   │
//!                     │  │   D      │   │
//!                     │  └──────────┘   │
//!                     └──────────────────┘
//! ```
//!
//! ### Grover Iteration (G = D · O)
//!
//! 1. **Oracle (O)**: Flip the phase of solution states
//!    - |x⟩ → (-1)^f(x) |x⟩
//!
//! 2. **Diffusion (D)**: "Inversion about the mean"
//!    - D = 2|ψ⟩⟨ψ| - I, where |ψ⟩ is uniform superposition
//!    - Amplifies marked states while suppressing unmarked ones
//!
//! ### Mathematical Analysis
//!
//! Let θ be defined by sin(θ) = √(M/N). After k iterations:
//! - Amplitude of marked states: sin((2k+1)θ)
//! - Optimal k ≈ (π/4)√(N/M)
//!
//! ## 📊 Computational Complexity
//!
//! - **Query Complexity**: O(√(N/M)) oracle queries
//! - **Gate Complexity**: O(n√N) for n-qubit system
//! - **Space Complexity**: O(n) qubits
//! - **Success Probability**: > 1 - M/N after optimal iterations
//!
//! ## 📍 Practical Applications
//!
//! 1. **Database Search**: Finding entries matching criteria
//! 2. **SAT Solving**: Finding satisfying assignments
//! 3. **Collision Finding**: Birthday attacks
//! 4. **Optimization**: Finding minima in unstructured spaces
//! 5. **Machine Learning**: Feature selection, model search
//!
//! ## 🔬 2025-26 Research Context
//!
//! - **Variational Quantum Search**: Hybrid Grover-VQE approaches
//! - **Grover Adaptive Search**: Self-adjusting iteration counts
//! - **Robust Grover**: Error-resilient implementations
//!
//! ## 📚 References
//!
//! - Grover, L. (1996). "A fast quantum mechanical algorithm for database search"
//! - Boyer et al. (1998). "Tight bounds on quantum searching"
//! - Nielsen & Chuang (2010). Ch. 6.1-6.2

use std::f64::consts::PI;

/// Oracle type for Grover's algorithm
///
/// Defines which items in the search space are "marked" (solutions)
#[derive(Debug, Clone)]
pub enum GroverOracle {
    /// Single marked element
    SingleTarget { target: usize },
    /// Multiple marked elements
    MultipleTargets { targets: Vec<usize> },
    /// Custom oracle defined by a function (for simulation)
    CustomFunction { evaluator: fn(usize) -> bool },
    /// Arbitrary SAT-like oracle (marks items satisfying constraints)
    SatOracle { clauses: Vec<Vec<i32>> },
}

impl GroverOracle {
    /// Checks if a given item is marked (is a solution)
    pub fn is_marked(&self, x: usize) -> bool {
        match self {
            GroverOracle::SingleTarget { target } => x == *target,
            GroverOracle::MultipleTargets { targets } => targets.contains(&x),
            GroverOracle::CustomFunction { evaluator } => evaluator(x),
            GroverOracle::SatOracle { clauses } => {
                // Check if assignment x satisfies all clauses
                clauses.iter().all(|clause| {
                    clause.iter().any(|&literal| {
                        let var = literal.unsigned_abs() as usize - 1;
                        let bit = (x >> var) & 1;
                        if literal > 0 { bit == 1 } else { bit == 0 }
                    })
                })
            }
        }
    }

    /// Returns the number of marked elements (if known)
    pub fn num_marked(&self, n: usize) -> Option<usize> {
        match self {
            GroverOracle::SingleTarget { .. } => Some(1),
            GroverOracle::MultipleTargets { targets } => Some(targets.len()),
            _ => {
                // Must count by enumeration
                let size = 1 << n;
                Some((0..size).filter(|&x| self.is_marked(x)).count())
            }
        }
    }
}

/// Quantum state for Grover's algorithm simulation
#[derive(Debug, Clone)]
pub struct GroverState {
    /// Number of qubits
    pub n: usize,
    /// Amplitude vector (2^n complex amplitudes, stored as real for simplicity)
    pub amplitudes: Vec<f64>,
}

impl GroverState {
    /// Creates initial uniform superposition |ψ⟩ = (1/√N) Σ|x⟩
    pub fn uniform_superposition(n: usize) -> Self {
        let size = 1 << n;
        let amp = 1.0 / (size as f64).sqrt();
        GroverState {
            n,
            amplitudes: vec![amp; size],
        }
    }

    /// Applies the Grover oracle (phase flip on marked states)
    ///
    /// |x⟩ → (-1)^f(x) |x⟩
    pub fn apply_oracle(&mut self, oracle: &GroverOracle) {
        for (x, amp) in self.amplitudes.iter_mut().enumerate() {
            if oracle.is_marked(x) {
                *amp = -*amp;  // Phase flip
            }
        }
    }

    /// Applies the Grover diffusion operator (inversion about mean)
    ///
    /// D = 2|ψ⟩⟨ψ| - I
    ///
    /// This reflects amplitudes about their mean, effectively
    /// amplifying marked states and suppressing unmarked ones.
    pub fn apply_diffusion(&mut self) {
        let size = self.amplitudes.len();
        
        // Compute mean amplitude
        let mean: f64 = self.amplitudes.iter().sum::<f64>() / (size as f64);
        
        // Reflect each amplitude about the mean
        // new_amp = 2*mean - old_amp
        for amp in &mut self.amplitudes {
            *amp = 2.0 * mean - *amp;
        }
    }

    /// Applies one complete Grover iteration: G = D · O
    pub fn apply_grover_iteration(&mut self, oracle: &GroverOracle) {
        self.apply_oracle(oracle);
        self.apply_diffusion();
    }

    /// Returns probability distribution over computational basis
    pub fn probabilities(&self) -> Vec<f64> {
        self.amplitudes.iter().map(|a| a * a).collect()
    }

    /// Returns the index with highest probability
    pub fn most_likely(&self) -> usize {
        self.amplitudes
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.abs().partial_cmp(&b.abs()).unwrap())
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    /// Returns probability of measuring a marked state
    pub fn success_probability(&self, oracle: &GroverOracle) -> f64 {
        self.amplitudes
            .iter()
            .enumerate()
            .filter(|(x, _)| oracle.is_marked(*x))
            .map(|(_, a)| a * a)
            .sum()
    }
}

/// Calculates the optimal number of Grover iterations
///
/// For N items with M marked, optimal iterations ≈ (π/4)√(N/M)
///
/// # Arguments
/// * `n` - Number of qubits (N = 2^n)
/// * `m` - Number of marked items
///
/// # Returns
/// Optimal iteration count
pub fn optimal_iterations(n: usize, m: usize) -> usize {
    let size = 1 << n;
    if m == 0 || m >= size {
        return 0;
    }
    
    let theta = (m as f64 / size as f64).sqrt().asin();
    let iterations = (PI / (4.0 * theta)).round() as usize;
    
    iterations.max(1)
}

/// Executes Grover's algorithm
///
/// # Arguments
/// * `oracle` - The marking oracle
/// * `n` - Number of qubits
/// * `iterations` - Number of Grover iterations (None = optimal)
///
/// # Returns
/// Tuple of (found_index, probability, state)
///
/// # Example
///
/// ```
/// use quantic_rust::algorithms::search::{GroverOracle, grovers_search};
///
/// // Search for item 42 in a 64-item space (6 qubits)
/// let oracle = GroverOracle::SingleTarget { target: 42 };
/// let (found, prob, _) = grovers_search(&oracle, 6, None);
/// 
/// assert_eq!(found, 42);
/// assert!(prob > 0.9);  // High success probability
/// ```
pub fn grovers_search(
    oracle: &GroverOracle,
    n: usize,
    iterations: Option<usize>,
) -> (usize, f64, GroverState) {
    // Initialize uniform superposition
    let mut state = GroverState::uniform_superposition(n);
    
    // Calculate optimal iterations if not specified
    let m = oracle.num_marked(n).unwrap_or(1);
    let num_iterations = iterations.unwrap_or_else(|| optimal_iterations(n, m));
    
    // Apply Grover iterations
    for _ in 0..num_iterations {
        state.apply_grover_iteration(oracle);
    }
    
    // Find most likely result
    let result = state.most_likely();
    let prob = state.amplitudes[result].powi(2);
    
    (result, prob, state)
}

/// Runs Grover's algorithm with execution trace for visualization
///
/// Returns intermediate states after each iteration
pub fn grovers_search_with_trace(
    oracle: &GroverOracle,
    n: usize,
    iterations: Option<usize>,
) -> (usize, f64, Vec<GroverState>) {
    let mut traces = Vec::new();
    let mut state = GroverState::uniform_superposition(n);
    traces.push(state.clone());
    
    let m = oracle.num_marked(n).unwrap_or(1);
    let num_iterations = iterations.unwrap_or_else(|| optimal_iterations(n, m));
    
    for _ in 0..num_iterations {
        state.apply_grover_iteration(oracle);
        traces.push(state.clone());
    }
    
    let result = state.most_likely();
    let prob = state.amplitudes[result].powi(2);
    
    (result, prob, traces)
}

/// Grover's algorithm for multiple solutions
///
/// When there are M solutions in a space of N items, this variant
/// is designed to find any one of them with high probability.
pub fn grovers_search_multiple(
    oracle: &GroverOracle,
    n: usize,
) -> Vec<(usize, f64)> {
    let (_, _, state) = grovers_search(oracle, n, None);
    
    // Return all marked items with their probabilities
    state.amplitudes
        .iter()
        .enumerate()
        .filter(|(x, _)| oracle.is_marked(*x))
        .map(|(x, a)| (x, a * a))
        .collect()
}

/// Creates a SAT oracle from DIMACS-style clauses
///
/// Each clause is a vector of literals where:
/// - Positive integer i means variable i is true
/// - Negative integer -i means variable i is false
///
/// # Example
///
/// ```
/// use quantic_rust::algorithms::search::create_sat_oracle;
///
/// // (x1 OR x2) AND (NOT x1 OR x3)
/// let clauses = vec![
///     vec![1, 2],     // x1 OR x2
///     vec![-1, 3],    // NOT x1 OR x3
/// ];
/// let oracle = create_sat_oracle(clauses);
/// ```
pub fn create_sat_oracle(clauses: Vec<Vec<i32>>) -> GroverOracle {
    GroverOracle::SatOracle { clauses }
}

/// Adaptive Grover search with unknown number of solutions
///
/// Uses exponentially increasing iteration counts to handle cases
/// where M is unknown. Based on Boyer et al. (1998).
///
/// # Arguments
/// * `oracle` - The marking oracle
/// * `n` - Number of qubits
/// * `max_attempts` - Maximum number of different iteration counts to try
///
/// # Returns
/// Found solution or None if not found within attempts
pub fn grovers_adaptive_search(
    oracle: &GroverOracle,
    n: usize,
    max_attempts: usize,
) -> Option<usize> {
    let size = 1 << n;
    let mut m_estimate = 1.0;
    
    for attempt in 0..max_attempts {
        // Random number of iterations based on current estimate
        let iterations = ((PI / 4.0) * (size as f64 / m_estimate).sqrt()) as usize;
        let iterations = iterations.max(1);
        
        let (result, _, _) = grovers_search(oracle, n, Some(iterations));
        
        if oracle.is_marked(result) {
            return Some(result);
        }
        
        // Increase estimate exponentially
        m_estimate *= 1.5_f64.powf(attempt as f64);
        if m_estimate > size as f64 {
            break;
        }
    }
    
    None
}

/// Circuit representation for Grover's algorithm
#[derive(Debug, Clone)]
pub struct GroverCircuit {
    /// Number of qubits
    pub n: usize,
    /// Number of iterations
    pub iterations: usize,
    /// Oracle description
    pub oracle_type: String,
}

impl GroverCircuit {
    /// Creates a Grover circuit specification
    pub fn new(n: usize, oracle: &GroverOracle, iterations: Option<usize>) -> Self {
        let m = oracle.num_marked(n).unwrap_or(1);
        let iters = iterations.unwrap_or_else(|| optimal_iterations(n, m));
        
        let oracle_type = match oracle {
            GroverOracle::SingleTarget { target } => format!("Single({})", target),
            GroverOracle::MultipleTargets { targets } => format!("Multi({} targets)", targets.len()),
            GroverOracle::CustomFunction { .. } => "Custom".to_string(),
            GroverOracle::SatOracle { clauses } => format!("SAT({} clauses)", clauses.len()),
        };
        
        GroverCircuit {
            n,
            iterations: iters,
            oracle_type,
        }
    }

    /// Returns circuit depth (approximate)
    pub fn depth(&self) -> usize {
        // Initial Hadamards + iterations * (oracle + diffuser)
        1 + self.iterations * (self.n + 2 * self.n)
    }

    /// Returns total gate count (approximate)
    pub fn gate_count(&self) -> usize {
        // n Hadamards + iterations * (oracle gates + diffuser gates)
        self.n + self.iterations * (2 * self.n + 3 * self.n)
    }

    /// Generates QASM representation (high-level)
    pub fn to_qasm(&self) -> String {
        format!(
            "// Grover's Search Circuit\n\
             // Qubits: {}\n\
             // Iterations: {}\n\
             // Oracle: {}\n\n\
             OPENQASM 2.0;\n\
             include \"qelib1.inc\";\n\
             qreg q[{}];\n\
             creg c[{}];\n\n\
             // Initial superposition\n\
             {}\n\n\
             // Grover iterations\n\
             // (Oracle and Diffusion gates would be specified here)\n\n\
             // Measurement\n\
             {}",
            self.n,
            self.iterations,
            self.oracle_type,
            self.n,
            self.n,
            (0..self.n).map(|i| format!("h q[{}];", i)).collect::<Vec<_>>().join("\n"),
            (0..self.n).map(|i| format!("measure q[{}] -> c[{}];", i, i)).collect::<Vec<_>>().join("\n"),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_target_search() {
        let oracle = GroverOracle::SingleTarget { target: 5 };
        let (found, prob, _) = grovers_search(&oracle, 4, None);
        
        assert_eq!(found, 5, "Should find the target");
        assert!(prob > 0.9, "Success probability should be high");
    }

    #[test]
    fn test_multiple_targets_search() {
        let oracle = GroverOracle::MultipleTargets { 
            targets: vec![3, 7, 11] 
        };
        let (found, prob, _) = grovers_search(&oracle, 4, None);
        
        assert!(oracle.is_marked(found), "Should find a marked item");
        assert!(prob > 0.2, "Individual probability should be reasonable");
    }

    #[test]
    fn test_optimal_iterations() {
        // For N=16 (4 qubits), M=1, optimal ≈ π/4 * √16 ≈ 3
        let k = optimal_iterations(4, 1);
        assert!(k >= 2 && k <= 4, "Optimal iterations for 4 qubits, 1 solution");
        
        // For N=64 (6 qubits), M=1, optimal ≈ π/4 * √64 ≈ 6
        let k = optimal_iterations(6, 1);
        assert!(k >= 5 && k <= 8, "Optimal iterations for 6 qubits, 1 solution");
    }

    #[test]
    fn test_diffusion_operator() {
        let mut state = GroverState::uniform_superposition(3);
        
        // After just diffusion (no oracle), state should remain uniform
        state.apply_diffusion();
        
        let first_amp = state.amplitudes[0];
        for amp in &state.amplitudes {
            assert!((amp - first_amp).abs() < 1e-10, "Should remain uniform");
        }
    }

    #[test]
    fn test_oracle_phase_flip() {
        let oracle = GroverOracle::SingleTarget { target: 2 };
        let mut state = GroverState::uniform_superposition(3);
        
        let original_amp_2 = state.amplitudes[2];
        state.apply_oracle(&oracle);
        
        // Target state should have flipped phase
        assert!((state.amplitudes[2] + original_amp_2).abs() < 1e-10);
        
        // Other states should be unchanged
        assert!((state.amplitudes[0] - state.amplitudes[1]).abs() < 1e-10);
    }

    #[test]
    fn test_sat_oracle() {
        // (x1 OR x2) AND (NOT x1 OR NOT x2) = XOR (exactly one of x1, x2)
        let clauses = vec![
            vec![1, 2],      // x1 OR x2
            vec![-1, -2],    // NOT x1 OR NOT x2
        ];
        let oracle = create_sat_oracle(clauses);
        
        // Solutions: 01 (1) and 10 (2)
        assert!(!oracle.is_marked(0));  // 00 - neither
        assert!(oracle.is_marked(1));   // 01 - x2 only
        assert!(oracle.is_marked(2));   // 10 - x1 only
        assert!(!oracle.is_marked(3));  // 11 - both
    }

    #[test]
    fn test_trace_execution() {
        let oracle = GroverOracle::SingleTarget { target: 3 };
        let (found, _, traces) = grovers_search_with_trace(&oracle, 3, Some(2));
        
        assert_eq!(found, 3);
        assert_eq!(traces.len(), 3);  // Initial + 2 iterations
        
        // Target amplitude should increase with iterations
        let initial_target_amp = traces[0].amplitudes[3].abs();
        let final_target_amp = traces[2].amplitudes[3].abs();
        assert!(final_target_amp > initial_target_amp);
    }

    #[test]
    fn test_adaptive_search() {
        let oracle = GroverOracle::SingleTarget { target: 10 };
        let result = grovers_adaptive_search(&oracle, 4, 10);
        
        assert_eq!(result, Some(10));
    }

    #[test]
    fn test_circuit_generation() {
        let oracle = GroverOracle::SingleTarget { target: 5 };
        let circuit = GroverCircuit::new(4, &oracle, None);
        
        assert_eq!(circuit.n, 4);
        assert!(circuit.iterations > 0);
        
        let qasm = circuit.to_qasm();
        assert!(qasm.contains("h q[0]"));
        assert!(qasm.contains("measure"));
    }
}
