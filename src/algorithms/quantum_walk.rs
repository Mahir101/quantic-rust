//! Quantum Walk Algorithms
//!
//! This module implements quantum random walk algorithms:
//! - Discrete-time quantum walks (coined walks)
//! - Continuous-time quantum walks
//! - Quantum walk on graphs
//! - Szegedy walks
//!
//! ## 🎯 Why is this used?
//! Quantum walks are the quantum version of classical random walks and provide a powerful 
//! framework for developing quantum algorithms. They are widely used for spatial search 
//! (Grover-like search on graphs), element distinctness, and evaluating Boolean formulas. 
//! They often provide quadratic speedups over classical random walks for hitting times and mixing times.
//!
//! ## ⚙️ How it works?
//! - **Discrete-Time (DTQW)**: Uses an auxiliary "coin" space. Each step consists of a 
//!   coin flip (unitary on the coin qubit) followed by a conditional shift operator that 
//!   moves the walker's position based on the coin's state.
//! - **Continuous-Time (CTQW)**: The evolution is governed by the Schrödinger equation 
//!   $e^{-iHt}$ where the Hamiltonian $H$ is the adjacency matrix or Laplacian of the graph.
//! - **Szegedy Walk**: A quantized version of Markov chains that maps a classical 
//!   stochastic transition matrix to a quantum walk on the edges of the graph using a 
//!   bipartite reflection scheme.
//!
//! ## 📍 Where to apply this?
//! - **Graph Search**: Finding a marked node in a structured database or graph.
//! - **Property Testing**: Determining if a graph is connected or bipartite.
//! - **Quantum Simulation**: Simulating transport phenomena in physical systems.
//! - **Optimization**: Using quantum walk-based simulated annealing.
//!
//! ## 📊 Code Behavior
//! - **Complexity**: 
//!     - DTQW: $O(1)$ per step for coin and $O(\log N)$ for conditional shifts.
//!     - CTQW: Simulated using $k$-th order Trotter-Suzuki, leading to $O(Poly(1/\epsilon))$ depth.
//! - **Connectivity**: Requires careful mapping of graph adjacency to qubit-to-qubit 
//!   entangling gates.
//! - **Side Effects**: Entanglement between the coin and position registers is fundamental 
//!   to the walk's non-classical behavior (interference).

use crate::gates::core::Gate;

// ============================================================================
// DISCRETE-TIME QUANTUM WALK (COINED WALK)
// ============================================================================

/// Coin operators for discrete quantum walks
#[derive(Clone, Debug)]
pub enum CoinOperator {
    /// Hadamard coin (balanced)
    Hadamard,
    /// Grover diffusion coin
    Grover,
    /// Discrete Fourier Transform coin
    DFT,
    /// Biased coin with parameter
    Biased(f64),
}

/// Generate a single step of discrete-time quantum walk on a line
/// 
/// The walk uses a coin qubit and position register.
/// Step = Shift · (Coin ⊗ I)
pub fn dtqw_step_line(
    coin_qubit: usize,
    position_qubits: &[usize],
    coin: &CoinOperator,
) -> Vec<Gate> {
    let mut gates = Vec::new();
    
    // Apply coin operator
    gates.extend(coin_operator(coin_qubit, coin));
    
    // Conditional shift based on coin state
    // If coin = |0⟩: shift left (subtract 1)
    // If coin = |1⟩: shift right (add 1)
    
    let n = position_qubits.len();
    
    // Shift right when coin is |1⟩
    for i in (0..n).rev() {
        if i == 0 {
            gates.push(Gate::CX(coin_qubit, position_qubits[0]));
        } else {
            // Controlled increment with carry
            let mut controls: Vec<usize> = vec![coin_qubit];
            controls.extend(&position_qubits[..i]);
            gates.push(Gate::MCX(controls, position_qubits[i]));
        }
    }
    
    // Shift left when coin is |0⟩
    gates.push(Gate::X(coin_qubit));
    for i in (0..n).rev() {
        if i == 0 {
            gates.push(Gate::CX(coin_qubit, position_qubits[0]));
        } else {
            let mut controls: Vec<usize> = vec![coin_qubit];
            controls.extend(&position_qubits[..i]);
            gates.push(Gate::MCX(controls, position_qubits[i]));
        }
    }
    gates.push(Gate::X(coin_qubit));
    
    gates
}

fn coin_operator(coin_qubit: usize, coin: &CoinOperator) -> Vec<Gate> {
    match coin {
        CoinOperator::Hadamard => vec![Gate::H(coin_qubit)],
        CoinOperator::Grover => vec![
            Gate::H(coin_qubit),
            Gate::Z(coin_qubit),
            Gate::H(coin_qubit),
        ],
        CoinOperator::DFT => vec![Gate::H(coin_qubit)],
        CoinOperator::Biased(theta) => vec![Gate::RY(coin_qubit, *theta)],
    }
}

/// Generate multi-step quantum walk circuit
pub fn dtqw_circuit(
    coin_qubit: usize,
    position_qubits: &[usize],
    coin: &CoinOperator,
    steps: usize,
) -> Vec<Gate> {
    let mut gates = Vec::new();
    
    for _ in 0..steps {
        gates.extend(dtqw_step_line(coin_qubit, position_qubits, coin));
    }
    
    gates
}

// ============================================================================
// CONTINUOUS-TIME QUANTUM WALK (CTQW)
// ============================================================================

/// Approximate continuous-time quantum walk using Trotter-Suzuki
/// 
/// The evolution e^(-iHt) where H is the adjacency matrix.
pub fn ctqw_trotter(
    adjacency: &[Vec<bool>],
    node_qubits: &[usize],
    time: f64,
    trotter_steps: usize,
) -> Vec<Gate> {
    let n = adjacency.len();
    let dt = time / trotter_steps as f64;
    
    let mut gates = Vec::new();
    
    for _ in 0..trotter_steps {
        for i in 0..n {
            for j in (i+1)..n {
                if adjacency[i][j] && i < node_qubits.len() && j < node_qubits.len() {
                    // XX + YY interaction for hopping e^(-i(XX+YY)dt)
                    gates.push(Gate::RXX(node_qubits[i], node_qubits[j], dt));
                    gates.push(Gate::RYY(node_qubits[i], node_qubits[j], dt));
                }
            }
        }
    }
    
    gates
}

// ============================================================================
// SZEGEDY WALK
// ============================================================================

/// Szegedy quantum walk step reflection
pub fn szegedy_reflection(
    transition_matrix: &[Vec<f64>],
    node_qubits: &[usize],
    coin_qubits: &[usize],
) -> Vec<Gate> {
    let mut gates = Vec::new();
    let n = transition_matrix.len();

    // Prepare state |phi_i> = sum_j sqrt(P_ij) |i,j>
    for i in 0..n {
        for j in 0..n {
            if transition_matrix[i][j] > 0.0 {
                let angle = 2.0 * transition_matrix[i][j].sqrt().acos();
                if i < node_qubits.len() && j < coin_qubits.len() {
                    gates.push(Gate::CRY(node_qubits[i], coin_qubits[j], angle));
                }
            }
        }
    }
    
    // Reflection operator around the prepared state
    // R = 2|Phi><Phi| - I
    // Simplified representation
    for &q in coin_qubits {
        gates.push(Gate::H(q));
        gates.push(Gate::X(q));
    }
    
    if !coin_qubits.is_empty() {
        let controls: Vec<usize> = coin_qubits[..coin_qubits.len()-1].to_vec();
        let target = coin_qubits[coin_qubits.len()-1];
        gates.push(Gate::MCX(controls, target));
    }
    
    for &q in coin_qubits {
        gates.push(Gate::X(q));
        gates.push(Gate::H(q));
    }

    gates
}
