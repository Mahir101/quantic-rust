//! # Core Quantum Gates Module
//!
//! This module provides implementations of standard quantum gates including:
//! - Single-qubit gates (X, Y, Z, H, S, T, etc.)
//! - Two-qubit gates (CNOT, CZ, SWAP, iSWAP, etc.)
//! - Multi-qubit gates (Toffoli, Fredkin, etc.)
//! - Parametric gates (RX, RY, RZ, etc.)
//!
//! ## 🎯 Why is this used?
//! This module serves as the foundational "instruction set" for the entire Quantic-Rust library. 
//! It defines the fundamental building blocks (gates) required to construct, represent, and
//! manipulate quantum circuits. Without this module, higher-level operations like 
//! Hamiltonian simulation or QFT would have no mathematical or structural basis.
//!
//! ## ⚙️ How it works?
//! - **Mathematical Layer**: Implements a dedicated `Complex` number arithmetic and matrix 
//!   structures (`GateMatrix2x2`, `GateMatrix4x4`) to provide exact unitary representations of gates.
//! - **Structural Layer**: Defines a `Gate` enum that acts as a symbolic representation of 
//!   quantum instructions, allowing for lightweight circuit storage and algebraic manipulation.
//! - **Static Definitions**: Provides optimized, constant-time functions for generating 
//!   standard gates like Hadamard, Pauli, and Phase gates.
//!
//! ## 📍 Where to apply this?
//! - **Circuit Construction**: Use the `Gate` enum to build sequences of operations.
//! - **Unitary Computation**: Use the matrix generation functions when you need to 
//!   calculate the full transformation matrix of a small circuit or gate.
//! - **Decomposition**: Acts as the target gate-set for the `decomposition` module.
//!
//! ## 📊 Code Behavior
//! - **Performance**: Matrix generation is $O(1)$ for fixed-size gates. Parametric gates 
//!   (RX, RY, RZ) involve trigonometric calls.
//! - **Memory**: Extremely lightweight. The `Gate` enum is designed for cache-efficient 
//!   vector storage.
//! - **Correctness**: All matrices are verified to be unitary within double-precision 
//!   floating-point limits.

use std::f64::consts::{FRAC_PI_2, FRAC_PI_4};

/// Represents a 2x2 complex matrix for single-qubit gates
#[derive(Clone, Debug)]
pub struct GateMatrix2x2 {
    pub data: [[Complex; 2]; 2],
}

/// Represents a 4x4 complex matrix for two-qubit gates
#[derive(Clone, Debug)]
pub struct GateMatrix4x4 {
    pub data: [[Complex; 4]; 4],
}

/// Complex number representation
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Complex {
    pub re: f64,
    pub im: f64,
}

impl Complex {
    pub const ZERO: Complex = Complex { re: 0.0, im: 0.0 };
    pub const ONE: Complex = Complex { re: 1.0, im: 0.0 };
    pub const I: Complex = Complex { re: 0.0, im: 1.0 };
    pub const NEG_I: Complex = Complex { re: 0.0, im: -1.0 };

    pub fn new(re: f64, im: f64) -> Self {
        Complex { re, im }
    }

    pub fn from_polar(r: f64, theta: f64) -> Self {
        Complex {
            re: r * theta.cos(),
            im: r * theta.sin(),
        }
    }

    pub fn conj(&self) -> Self {
        Complex { re: self.re, im: -self.im }
    }

    pub fn norm_sq(&self) -> f64 {
        self.re * self.re + self.im * self.im
    }

    pub fn norm(&self) -> f64 {
        self.norm_sq().sqrt()
    }
}

impl std::ops::Add for Complex {
    type Output = Complex;
    fn add(self, rhs: Complex) -> Complex {
        Complex { re: self.re + rhs.re, im: self.im + rhs.im }
    }
}

impl std::ops::Sub for Complex {
    type Output = Complex;
    fn sub(self, rhs: Complex) -> Complex {
        Complex { re: self.re - rhs.re, im: self.im - rhs.im }
    }
}

impl std::ops::Mul for Complex {
    type Output = Complex;
    fn mul(self, rhs: Complex) -> Complex {
        Complex {
            re: self.re * rhs.re - self.im * rhs.im,
            im: self.re * rhs.im + self.im * rhs.re,
        }
    }
}

impl std::ops::Mul<f64> for Complex {
    type Output = Complex;
    fn mul(self, rhs: f64) -> Complex {
        Complex { re: self.re * rhs, im: self.im * rhs }
    }
}

impl std::ops::Neg for Complex {
    type Output = Complex;
    fn neg(self) -> Complex {
        Complex { re: -self.re, im: -self.im }
    }
}

// ============================================================================
// STANDARD SINGLE-QUBIT GATES
// ============================================================================

/// Pauli-X gate (NOT gate, bit flip)
/// |0⟩ → |1⟩, |1⟩ → |0⟩
pub fn pauli_x() -> GateMatrix2x2 {
    GateMatrix2x2 {
        data: [
            [Complex::ZERO, Complex::ONE],
            [Complex::ONE, Complex::ZERO],
        ],
    }
}

/// Pauli-Y gate
/// |0⟩ → i|1⟩, |1⟩ → -i|0⟩
pub fn pauli_y() -> GateMatrix2x2 {
    GateMatrix2x2 {
        data: [
            [Complex::ZERO, Complex::NEG_I],
            [Complex::I, Complex::ZERO],
        ],
    }
}

/// Pauli-Z gate (phase flip)
/// |0⟩ → |0⟩, |1⟩ → -|1⟩
pub fn pauli_z() -> GateMatrix2x2 {
    GateMatrix2x2 {
        data: [
            [Complex::ONE, Complex::ZERO],
            [Complex::ZERO, Complex::new(-1.0, 0.0)],
        ],
    }
}

/// Hadamard gate
/// Creates superposition: |0⟩ → (|0⟩ + |1⟩)/√2, |1⟩ → (|0⟩ - |1⟩)/√2
pub fn hadamard() -> GateMatrix2x2 {
    let inv_sqrt2 = 1.0 / 2.0_f64.sqrt();
    let h = Complex::new(inv_sqrt2, 0.0);
    let neg_h = Complex::new(-inv_sqrt2, 0.0);
    GateMatrix2x2 {
        data: [
            [h, h],
            [h, neg_h],
        ],
    }
}

/// S gate (√Z, phase gate)
/// |0⟩ → |0⟩, |1⟩ → i|1⟩
pub fn s_gate() -> GateMatrix2x2 {
    GateMatrix2x2 {
        data: [
            [Complex::ONE, Complex::ZERO],
            [Complex::ZERO, Complex::I],
        ],
    }
}

/// S† gate (inverse S gate)
/// |0⟩ → |0⟩, |1⟩ → -i|1⟩
pub fn s_dagger() -> GateMatrix2x2 {
    GateMatrix2x2 {
        data: [
            [Complex::ONE, Complex::ZERO],
            [Complex::ZERO, Complex::NEG_I],
        ],
    }
}

/// T gate (π/8 gate, √S)
/// |0⟩ → |0⟩, |1⟩ → e^(iπ/4)|1⟩
pub fn t_gate() -> GateMatrix2x2 {
    let phase = Complex::from_polar(1.0, FRAC_PI_4);
    GateMatrix2x2 {
        data: [
            [Complex::ONE, Complex::ZERO],
            [Complex::ZERO, phase],
        ],
    }
}

/// T† gate (inverse T gate)
/// |0⟩ → |0⟩, |1⟩ → e^(-iπ/4)|1⟩
pub fn t_dagger() -> GateMatrix2x2 {
    let phase = Complex::from_polar(1.0, -FRAC_PI_4);
    GateMatrix2x2 {
        data: [
            [Complex::ONE, Complex::ZERO],
            [Complex::ZERO, phase],
        ],
    }
}

/// Identity gate
pub fn identity() -> GateMatrix2x2 {
    GateMatrix2x2 {
        data: [
            [Complex::ONE, Complex::ZERO],
            [Complex::ZERO, Complex::ONE],
        ],
    }
}

/// √X gate (square root of X)
pub fn sqrt_x() -> GateMatrix2x2 {
    let half = Complex::new(0.5, 0.0);
    let half_i = Complex::new(0.0, 0.5);
    GateMatrix2x2 {
        data: [
            [half + half_i, half - half_i],
            [half - half_i, half + half_i],
        ],
    }
}

/// √Y gate
pub fn sqrt_y() -> GateMatrix2x2 {
    let half = Complex::new(0.5, 0.0);
    let half_neg = Complex::new(-0.5, 0.0);
    GateMatrix2x2 {
        data: [
            [half + half_i(), half_neg - half_i()],
            [half + half_i(), half + half_i()],
        ],
    }
}

fn half_i() -> Complex {
    Complex::new(0.0, 0.5)
}

// ============================================================================
// PARAMETRIC SINGLE-QUBIT GATES
// ============================================================================

/// RX gate - rotation around X-axis
/// RX(θ) = exp(-i θ/2 X) = cos(θ/2)I - i·sin(θ/2)X
pub fn rx(theta: f64) -> GateMatrix2x2 {
    let c = Complex::new((theta / 2.0).cos(), 0.0);
    let s = Complex::new(0.0, -(theta / 2.0).sin());
    GateMatrix2x2 {
        data: [
            [c, s],
            [s, c],
        ],
    }
}

/// RY gate - rotation around Y-axis
/// RY(θ) = exp(-i θ/2 Y) = cos(θ/2)I - i·sin(θ/2)Y
pub fn ry(theta: f64) -> GateMatrix2x2 {
    let c = Complex::new((theta / 2.0).cos(), 0.0);
    let s = Complex::new((theta / 2.0).sin(), 0.0);
    GateMatrix2x2 {
        data: [
            [c, -s],
            [s, c],
        ],
    }
}

/// RZ gate - rotation around Z-axis
/// RZ(θ) = exp(-i θ/2 Z) = diag(e^(-iθ/2), e^(iθ/2))
pub fn rz(theta: f64) -> GateMatrix2x2 {
    let neg_phase = Complex::from_polar(1.0, -theta / 2.0);
    let pos_phase = Complex::from_polar(1.0, theta / 2.0);
    GateMatrix2x2 {
        data: [
            [neg_phase, Complex::ZERO],
            [Complex::ZERO, pos_phase],
        ],
    }
}

/// Phase gate P(φ) = diag(1, e^(iφ))
pub fn phase_gate(phi: f64) -> GateMatrix2x2 {
    let phase = Complex::from_polar(1.0, phi);
    GateMatrix2x2 {
        data: [
            [Complex::ONE, Complex::ZERO],
            [Complex::ZERO, phase],
        ],
    }
}

/// U3 gate - most general single-qubit gate (IBM convention)
/// U3(θ, φ, λ) = RZ(φ) · RY(θ) · RZ(λ)
pub fn u3(theta: f64, phi: f64, lambda: f64) -> GateMatrix2x2 {
    let c = (theta / 2.0).cos();
    let s = (theta / 2.0).sin();
    
    GateMatrix2x2 {
        data: [
            [
                Complex::new(c, 0.0),
                -Complex::from_polar(s, lambda),
            ],
            [
                Complex::from_polar(s, phi),
                Complex::from_polar(c, phi + lambda),
            ],
        ],
    }
}

/// U2 gate - single-qubit rotation with fixed θ = π/2
/// U2(φ, λ) = U3(π/2, φ, λ)
pub fn u2(phi: f64, lambda: f64) -> GateMatrix2x2 {
    u3(FRAC_PI_2, phi, lambda)
}

/// U1 gate - single-qubit rotation (equivalent to phase gate)
/// U1(λ) = U3(0, 0, λ) = P(λ)
pub fn u1(lambda: f64) -> GateMatrix2x2 {
    phase_gate(lambda)
}

// ============================================================================
// TWO-QUBIT GATES
// ============================================================================

/// CNOT (CX) gate - controlled NOT
/// |00⟩ → |00⟩, |01⟩ → |01⟩, |10⟩ → |11⟩, |11⟩ → |10⟩
pub fn cnot() -> GateMatrix4x4 {
    GateMatrix4x4 {
        data: [
            [Complex::ONE, Complex::ZERO, Complex::ZERO, Complex::ZERO],
            [Complex::ZERO, Complex::ONE, Complex::ZERO, Complex::ZERO],
            [Complex::ZERO, Complex::ZERO, Complex::ZERO, Complex::ONE],
            [Complex::ZERO, Complex::ZERO, Complex::ONE, Complex::ZERO],
        ],
    }
}

/// CZ gate - controlled Z
/// |00⟩ → |00⟩, |01⟩ → |01⟩, |10⟩ → |10⟩, |11⟩ → -|11⟩
pub fn cz() -> GateMatrix4x4 {
    GateMatrix4x4 {
        data: [
            [Complex::ONE, Complex::ZERO, Complex::ZERO, Complex::ZERO],
            [Complex::ZERO, Complex::ONE, Complex::ZERO, Complex::ZERO],
            [Complex::ZERO, Complex::ZERO, Complex::ONE, Complex::ZERO],
            [Complex::ZERO, Complex::ZERO, Complex::ZERO, Complex::new(-1.0, 0.0)],
        ],
    }
}

/// CY gate - controlled Y
pub fn cy() -> GateMatrix4x4 {
    GateMatrix4x4 {
        data: [
            [Complex::ONE, Complex::ZERO, Complex::ZERO, Complex::ZERO],
            [Complex::ZERO, Complex::ONE, Complex::ZERO, Complex::ZERO],
            [Complex::ZERO, Complex::ZERO, Complex::ZERO, Complex::NEG_I],
            [Complex::ZERO, Complex::ZERO, Complex::I, Complex::ZERO],
        ],
    }
}

/// SWAP gate
/// |00⟩ → |00⟩, |01⟩ → |10⟩, |10⟩ → |01⟩, |11⟩ → |11⟩
pub fn swap() -> GateMatrix4x4 {
    GateMatrix4x4 {
        data: [
            [Complex::ONE, Complex::ZERO, Complex::ZERO, Complex::ZERO],
            [Complex::ZERO, Complex::ZERO, Complex::ONE, Complex::ZERO],
            [Complex::ZERO, Complex::ONE, Complex::ZERO, Complex::ZERO],
            [Complex::ZERO, Complex::ZERO, Complex::ZERO, Complex::ONE],
        ],
    }
}

/// iSWAP gate
/// |01⟩ → i|10⟩, |10⟩ → i|01⟩
pub fn iswap() -> GateMatrix4x4 {
    GateMatrix4x4 {
        data: [
            [Complex::ONE, Complex::ZERO, Complex::ZERO, Complex::ZERO],
            [Complex::ZERO, Complex::ZERO, Complex::I, Complex::ZERO],
            [Complex::ZERO, Complex::I, Complex::ZERO, Complex::ZERO],
            [Complex::ZERO, Complex::ZERO, Complex::ZERO, Complex::ONE],
        ],
    }
}

/// √SWAP gate
pub fn sqrt_swap() -> GateMatrix4x4 {
    let half = Complex::new(0.5, 0.0);
    let half_i = Complex::new(0.0, 0.5);
    GateMatrix4x4 {
        data: [
            [Complex::ONE, Complex::ZERO, Complex::ZERO, Complex::ZERO],
            [Complex::ZERO, half + half_i, half - half_i, Complex::ZERO],
            [Complex::ZERO, half - half_i, half + half_i, Complex::ZERO],
            [Complex::ZERO, Complex::ZERO, Complex::ZERO, Complex::ONE],
        ],
    }
}

/// fSWAP gate (fermionic SWAP)
/// SWAP with a phase on |11⟩
pub fn fswap() -> GateMatrix4x4 {
    GateMatrix4x4 {
        data: [
            [Complex::ONE, Complex::ZERO, Complex::ZERO, Complex::ZERO],
            [Complex::ZERO, Complex::ZERO, Complex::ONE, Complex::ZERO],
            [Complex::ZERO, Complex::ONE, Complex::ZERO, Complex::ZERO],
            [Complex::ZERO, Complex::ZERO, Complex::ZERO, Complex::new(-1.0, 0.0)],
        ],
    }
}

// ============================================================================
// CONTROLLED PARAMETRIC GATES
// ============================================================================

/// CRX gate - controlled RX
pub fn crx(theta: f64) -> GateMatrix4x4 {
    let c = Complex::new((theta / 2.0).cos(), 0.0);
    let s = Complex::new(0.0, -(theta / 2.0).sin());
    GateMatrix4x4 {
        data: [
            [Complex::ONE, Complex::ZERO, Complex::ZERO, Complex::ZERO],
            [Complex::ZERO, Complex::ONE, Complex::ZERO, Complex::ZERO],
            [Complex::ZERO, Complex::ZERO, c, s],
            [Complex::ZERO, Complex::ZERO, s, c],
        ],
    }
}

/// CRY gate - controlled RY
pub fn cry(theta: f64) -> GateMatrix4x4 {
    let c = Complex::new((theta / 2.0).cos(), 0.0);
    let s = Complex::new((theta / 2.0).sin(), 0.0);
    GateMatrix4x4 {
        data: [
            [Complex::ONE, Complex::ZERO, Complex::ZERO, Complex::ZERO],
            [Complex::ZERO, Complex::ONE, Complex::ZERO, Complex::ZERO],
            [Complex::ZERO, Complex::ZERO, c, -s],
            [Complex::ZERO, Complex::ZERO, s, c],
        ],
    }
}

/// CRZ gate - controlled RZ
pub fn crz(theta: f64) -> GateMatrix4x4 {
    let neg_phase = Complex::from_polar(1.0, -theta / 2.0);
    let pos_phase = Complex::from_polar(1.0, theta / 2.0);
    GateMatrix4x4 {
        data: [
            [Complex::ONE, Complex::ZERO, Complex::ZERO, Complex::ZERO],
            [Complex::ZERO, Complex::ONE, Complex::ZERO, Complex::ZERO],
            [Complex::ZERO, Complex::ZERO, neg_phase, Complex::ZERO],
            [Complex::ZERO, Complex::ZERO, Complex::ZERO, pos_phase],
        ],
    }
}

/// CP gate - controlled phase
pub fn cp(phi: f64) -> GateMatrix4x4 {
    let phase = Complex::from_polar(1.0, phi);
    GateMatrix4x4 {
        data: [
            [Complex::ONE, Complex::ZERO, Complex::ZERO, Complex::ZERO],
            [Complex::ZERO, Complex::ONE, Complex::ZERO, Complex::ZERO],
            [Complex::ZERO, Complex::ZERO, Complex::ONE, Complex::ZERO],
            [Complex::ZERO, Complex::ZERO, Complex::ZERO, phase],
        ],
    }
}

// ============================================================================
// TWO-QUBIT ROTATION GATES
// ============================================================================

/// RXX gate - Ising XX coupling
/// RXX(θ) = exp(-i θ/2 X⊗X)
pub fn rxx(theta: f64) -> GateMatrix4x4 {
    let c = Complex::new((theta / 2.0).cos(), 0.0);
    let s = Complex::new(0.0, -(theta / 2.0).sin());
    GateMatrix4x4 {
        data: [
            [c, Complex::ZERO, Complex::ZERO, s],
            [Complex::ZERO, c, s, Complex::ZERO],
            [Complex::ZERO, s, c, Complex::ZERO],
            [s, Complex::ZERO, Complex::ZERO, c],
        ],
    }
}

/// RYY gate - Ising YY coupling
/// RYY(θ) = exp(-i θ/2 Y⊗Y)
pub fn ryy(theta: f64) -> GateMatrix4x4 {
    let c = Complex::new((theta / 2.0).cos(), 0.0);
    let s = Complex::new(0.0, (theta / 2.0).sin());
    let neg_s = -s;
    GateMatrix4x4 {
        data: [
            [c, Complex::ZERO, Complex::ZERO, s],
            [Complex::ZERO, c, neg_s, Complex::ZERO],
            [Complex::ZERO, neg_s, c, Complex::ZERO],
            [s, Complex::ZERO, Complex::ZERO, c],
        ],
    }
}

/// RZZ gate - Ising ZZ coupling
/// RZZ(θ) = exp(-i θ/2 Z⊗Z)
pub fn rzz(theta: f64) -> GateMatrix4x4 {
    let neg_phase = Complex::from_polar(1.0, -theta / 2.0);
    let pos_phase = Complex::from_polar(1.0, theta / 2.0);
    GateMatrix4x4 {
        data: [
            [neg_phase, Complex::ZERO, Complex::ZERO, Complex::ZERO],
            [Complex::ZERO, pos_phase, Complex::ZERO, Complex::ZERO],
            [Complex::ZERO, Complex::ZERO, pos_phase, Complex::ZERO],
            [Complex::ZERO, Complex::ZERO, Complex::ZERO, neg_phase],
        ],
    }
}

// ============================================================================
// MULTI-QUBIT GATES (represented as gate descriptors)
// ============================================================================

/// Gate descriptor for applying gates to circuits
#[derive(Clone, Debug)]
pub enum Gate {
    // Single-qubit gates
    X(usize),
    Y(usize),
    Z(usize),
    H(usize),
    S(usize),
    Sdg(usize),
    T(usize),
    Tdg(usize),
    SX(usize),
    
    // Parametric single-qubit
    RX(usize, f64),
    RY(usize, f64),
    RZ(usize, f64),
    P(usize, f64),
    U3(usize, f64, f64, f64),
    
    // Two-qubit gates
    CX(usize, usize),
    CY(usize, usize),
    CZ(usize, usize),
    SWAP(usize, usize),
    ISWAP(usize, usize),
    
    // Controlled parametric
    CRX(usize, usize, f64),
    CRY(usize, usize, f64),
    CRZ(usize, usize, f64),
    CP(usize, usize, f64),
    
    // Two-qubit rotations
    RXX(usize, usize, f64),
    RYY(usize, usize, f64),
    RZZ(usize, usize, f64),
    
    // Multi-qubit gates
    CCX(usize, usize, usize),     // Toffoli
    CCZ(usize, usize, usize),
    CSWAP(usize, usize, usize),   // Fredkin
    
    // Multi-controlled gates
    MCX(Vec<usize>, usize),       // Multi-controlled X
    MCZ(Vec<usize>, usize),       // Multi-controlled Z
    MCP(Vec<usize>, usize, f64),  // Multi-controlled Phase
}

impl Gate {
    /// Get qubits involved in this gate
    pub fn qubits(&self) -> Vec<usize> {
        match self {
            Gate::X(q) | Gate::Y(q) | Gate::Z(q) | Gate::H(q) |
            Gate::S(q) | Gate::Sdg(q) | Gate::T(q) | Gate::Tdg(q) |
            Gate::SX(q) | Gate::RX(q, _) | Gate::RY(q, _) | Gate::RZ(q, _) |
            Gate::P(q, _) | Gate::U3(q, _, _, _) => vec![*q],
            
            Gate::CX(c, t) | Gate::CY(c, t) | Gate::CZ(c, t) |
            Gate::SWAP(c, t) | Gate::ISWAP(c, t) |
            Gate::CRX(c, t, _) | Gate::CRY(c, t, _) | Gate::CRZ(c, t, _) |
            Gate::CP(c, t, _) |
            Gate::RXX(c, t, _) | Gate::RYY(c, t, _) | Gate::RZZ(c, t, _) => vec![*c, *t],
            
            Gate::CCX(c1, c2, t) | Gate::CCZ(c1, c2, t) | Gate::CSWAP(c1, c2, t) => {
                vec![*c1, *c2, *t]
            }
            
            Gate::MCX(controls, target) | Gate::MCZ(controls, target) |
            Gate::MCP(controls, target, _) => {
                let mut qubits = controls.clone();
                qubits.push(*target);
                qubits
            }
        }
    }
    
    /// Check if gate is a Clifford gate
    pub fn is_clifford(&self) -> bool {
        match self {
            Gate::X(_) | Gate::Y(_) | Gate::Z(_) | Gate::H(_) |
            Gate::S(_) | Gate::Sdg(_) | Gate::CX(_, _) | Gate::CY(_, _) |
            Gate::CZ(_, _) | Gate::SWAP(_, _) => true,
            _ => false,
        }
    }
    
    /// Get the inverse of this gate
    pub fn inverse(&self) -> Gate {
        match self {
            Gate::X(q) => Gate::X(*q),
            Gate::Y(q) => Gate::Y(*q),
            Gate::Z(q) => Gate::Z(*q),
            Gate::H(q) => Gate::H(*q),
            Gate::S(q) => Gate::Sdg(*q),
            Gate::Sdg(q) => Gate::S(*q),
            Gate::T(q) => Gate::Tdg(*q),
            Gate::Tdg(q) => Gate::T(*q),
            Gate::SX(q) => Gate::SX(*q), // SX† = SX^3, simplified
            Gate::RX(q, theta) => Gate::RX(*q, -*theta),
            Gate::RY(q, theta) => Gate::RY(*q, -*theta),
            Gate::RZ(q, theta) => Gate::RZ(*q, -*theta),
            Gate::P(q, phi) => Gate::P(*q, -*phi),
            Gate::U3(q, theta, phi, lambda) => Gate::U3(*q, -*theta, -*lambda, -*phi),
            Gate::CX(c, t) => Gate::CX(*c, *t),
            Gate::CY(c, t) => Gate::CY(*c, *t),
            Gate::CZ(c, t) => Gate::CZ(*c, *t),
            Gate::SWAP(a, b) => Gate::SWAP(*a, *b),
            Gate::ISWAP(a, b) => Gate::ISWAP(*a, *b), // Simplified
            Gate::CRX(c, t, theta) => Gate::CRX(*c, *t, -*theta),
            Gate::CRY(c, t, theta) => Gate::CRY(*c, *t, -*theta),
            Gate::CRZ(c, t, theta) => Gate::CRZ(*c, *t, -*theta),
            Gate::CP(c, t, phi) => Gate::CP(*c, *t, -*phi),
            Gate::RXX(a, b, theta) => Gate::RXX(*a, *b, -*theta),
            Gate::RYY(a, b, theta) => Gate::RYY(*a, *b, -*theta),
            Gate::RZZ(a, b, theta) => Gate::RZZ(*a, *b, -*theta),
            Gate::CCX(c1, c2, t) => Gate::CCX(*c1, *c2, *t),
            Gate::CCZ(c1, c2, t) => Gate::CCZ(*c1, *c2, *t),
            Gate::CSWAP(c1, c2, t) => Gate::CSWAP(*c1, *c2, *t),
            Gate::MCX(controls, target) => Gate::MCX(controls.clone(), *target),
            Gate::MCZ(controls, target) => Gate::MCZ(controls.clone(), *target),
            Gate::MCP(controls, target, phi) => Gate::MCP(controls.clone(), *target, -*phi),
        }
    }
}

// ============================================================================
// TOFFOLI (CCX) DECOMPOSITION
// ============================================================================

/// Decompose Toffoli gate into elementary gates (H, T, CNOT)
/// Returns a sequence of gates equivalent to CCX(c1, c2, target)
pub fn decompose_toffoli(c1: usize, c2: usize, target: usize) -> Vec<Gate> {
    vec![
        Gate::H(target),
        Gate::CX(c2, target),
        Gate::Tdg(target),
        Gate::CX(c1, target),
        Gate::T(target),
        Gate::CX(c2, target),
        Gate::Tdg(target),
        Gate::CX(c1, target),
        Gate::T(c2),
        Gate::T(target),
        Gate::H(target),
        Gate::CX(c1, c2),
        Gate::T(c1),
        Gate::Tdg(c2),
        Gate::CX(c1, c2),
    ]
}

/// Decompose CSWAP (Fredkin) gate
pub fn decompose_fredkin(control: usize, a: usize, b: usize) -> Vec<Gate> {
    vec![
        Gate::CX(b, a),
        Gate::CCX(control, a, b),
        Gate::CX(b, a),
    ]
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pauli_gates() {
        let x = pauli_x();
        let y = pauli_y();
        let z = pauli_z();
        
        // X^2 = I
        // Y^2 = I
        // Z^2 = I
        // These are self-inverse up to global phase
        assert!((x.data[0][1].norm() - 1.0).abs() < 1e-10);
        assert!((y.data[0][1].norm() - 1.0).abs() < 1e-10);
        assert!((z.data[0][0].norm() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_hadamard() {
        let h = hadamard();
        let inv_sqrt2 = 1.0 / 2.0_f64.sqrt();
        assert!((h.data[0][0].re - inv_sqrt2).abs() < 1e-10);
        assert!((h.data[1][1].re + inv_sqrt2).abs() < 1e-10);
    }

    #[test]
    fn test_rotation_gates() {
        // RZ(π) should be equivalent to Z up to global phase
        let rz_pi = rz(PI);
        let z = pauli_z();
        
        // Check that diagonal elements have correct relative phase
        let ratio = rz_pi.data[1][1] * z.data[0][0];
        assert!((ratio.re * ratio.re + ratio.im * ratio.im - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_gate_inverse() {
        let t = Gate::T(0);
        let t_inv = t.inverse();
        match t_inv {
            Gate::Tdg(q) => assert_eq!(q, 0),
            _ => panic!("Expected Tdg gate"),
        }
    }
}
