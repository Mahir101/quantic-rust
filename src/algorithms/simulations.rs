//! Quantum Simulation Algorithms
//!
//! This module implements techniques for simulating quantum systems:
//! - Trotter-Suzuki decomposition
//! - Linear Combination of Unitaries (LCU)
//! - Hamiltonian simulation
//!
//! ## 🎯 Why is this used?
//! Simulating the dynamics of quantum systems (Hamiltonian simulation) is the "killer app" 
//! of quantum computing. It allows for the study of complex chemical reactions, material 
//! properties, and high-energy physics that are classically intractable. These routines 
//! are essential for calculating ground state energies, reaction rates, and time-dependent 
//! observables in molecular and condensed matter systems.
//!
//! ## ⚙️ How it works?
//! - **Trotter-Suzuki Decomposition**: Approximates the evolution $e^{-iHt}$ for a 
//!   Hamiltonian $H = \sum H_j$ by interleaving the individual evolutions $e^{-iH_j \Delta t}$ 
//!   over small time steps. Higher-order formulas (ST2, ST4) provide better accuracy by 
//!   symmetrizing the gate sequence.
//! - **Linear Combination of Unitaries (LCU)**: Represents the operator as a sum of 
//!   unitaries $\sum \alpha_j U_j$. It uses a `PREPARE` circuit to load coefficients into an 
//!   ancilla register and a `SELECT` circuit to apply the corresponding unitary $U_j$.
//!
//! ## 📍 Where to apply this?
//! - **Quantum Chemistry**: Simulating electronic structure Hamiltonians (e.g., in VQE or QPE).
//! - **Solid State Physics**: Modeling the Hubbard model or other lattice Hamiltonians.
//! - **Algorithm Design**: As a building block for HHL (which requires simulating $e^{iAt}$).
//!
//! ## 📊 Code Behavior
//! - **Complexity**: 
//!     - Trotter: Depends on the norm of the Hamiltonian terms and required precision $\epsilon$.
//!     - LCU: Complexity is typically linear in the sum of coefficients $\sum |\alpha_j|$.
//! - **Interaction**: Requires parameterized gates (like RXX, RYY, RZZ) or pre-decomposed 
//!   circuits for each term.
//! - **Precision**: Trotter error scales as $(\Delta t)^k$; error in LCU is handled via 
//!   amplitude amplification or oblivious amplitude amplification.

use crate::gates::core::Gate;

// ============================================================================
// TROTTER-SUZUKI DECOMPOSITION
// ============================================================================

/// First-order Trotter-Suzuki decomposition
/// e^(-i (H1 + H2 + ... + Hk) t) ≈ (e^(-i H1 dt) e^(-i H2 dt) ... e^(-i Hk dt))^n
pub fn trotter_first_order(
    hamiltonian_terms: &[Vec<Gate>],
    time: f64,
    steps: usize,
) -> Vec<Gate> {
    let mut gates = Vec::new();
    let _dt = time / steps as f64;
    
    for _ in 0..steps {
        for term in hamiltonian_terms {
            // Here each term is assumed to be the evolution e^(-i H_j dt)
            gates.extend(term.iter().cloned());
        }
    }
    
    gates
}

/// Second-order Trotter-Suzuki decomposition (ST2)
/// e^(-i (H1 + H2) t) ≈ e^(-i H1 dt/2) e^(-i H2 dt) e^(-i H1 dt/2)
pub fn trotter_second_order(
    h1_evolution: &[Gate],
    h2_evolution: &[Gate],
    time: f64,
    steps: usize,
) -> Vec<Gate> {
    let mut gates = Vec::new();
    let _dt = time / steps as f64;
    
    // Half-step versions would be needed here (adjusting angles in gates)
    // For this implementation, we assume the input slices are pre-scaled or we scale them
    
    for _ in 0..steps {
        gates.extend(h1_evolution.iter().cloned());
        gates.extend(h2_evolution.iter().cloned());
        gates.extend(h1_evolution.iter().cloned());
    }
    
    gates
}

// ============================================================================
// LINEAR COMBINATION OF UNITARIES (LCU)
// ============================================================================

/// LCU Step: SELECT and PREPARE operators
/// 
/// H = sum_j alpha_j U_j
pub fn lcu_simulation(
    prepare: &[Gate],
    select: &[Gate],
    steps: usize,
) -> Vec<Gate> {
    let mut gates = Vec::new();
    
    for _ in 0..steps {
        gates.extend(prepare.iter().cloned());
        gates.extend(select.iter().cloned());
        // Inverse PREPARE
        let inv_prepare: Vec<Gate> = prepare.iter().rev().map(|g| g.inverse()).collect();
        gates.extend(inv_prepare);
    }
    
    gates
}
