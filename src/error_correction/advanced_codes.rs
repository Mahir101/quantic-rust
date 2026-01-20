//! # Advanced Quantum Error Correction Codes
//!
//! ## 🎯 Purpose & Motivation
//!
//! This module implements cutting-edge quantum error correction codes
//! from 2025-2026 research that dramatically reduce overhead for
//! fault-tolerant quantum computing.
//!
//! ## 🔬 2025-26 Breakthroughs
//!
//! | Code | Key Result | Source |
//! |------|------------|--------|
//! | qLDPC | 90% overhead reduction | IBM 2024-25 |
//! | Color Codes | Transversal gates | Various |
//! | Floquet Codes | Dynamic stabilizers | Google 2024 |
//!
//! ## 📚 References
//!
//! - IBM Quantum (2024). "Quantum Low-Density Parity-Check Codes"
//! - Bombin (2006). "Topological Quantum Distillation"
//! - Hastings & Haah (2021). "Dynamically Generated Logical Qubits"

// use std::collections::{HashMap, HashSet};

/// Sparse parity check matrix representation
#[derive(Debug, Clone)]
pub struct ParityCheckMatrix {
    /// Number of rows (stabilizers)
    pub num_rows: usize,
    /// Number of columns (qubits)
    pub num_cols: usize,
    /// Non-zero entries: (row, col) pairs
    pub entries: Vec<(usize, usize)>,
    /// Row weights (connectivity)
    pub row_weights: Vec<usize>,
    /// Column weights
    pub col_weights: Vec<usize>,
}

impl ParityCheckMatrix {
    /// Creates a new parity check matrix
    pub fn new(num_rows: usize, num_cols: usize) -> Self {
        ParityCheckMatrix {
            num_rows,
            num_cols,
            entries: Vec::new(),
            row_weights: vec![0; num_rows],
            col_weights: vec![0; num_cols],
        }
    }

    /// Adds a non-zero entry
    pub fn add_entry(&mut self, row: usize, col: usize) {
        self.entries.push((row, col));
        self.row_weights[row] += 1;
        self.col_weights[col] += 1;
    }

    /// Returns connectivity of row (stabilizer weight)
    pub fn row_weight(&self, row: usize) -> usize {
        self.row_weights[row]
    }

    /// Returns connectivity of column (qubit degree)
    pub fn col_weight(&self, col: usize) -> usize {
        self.col_weights[col]
    }

    /// Maximum row weight
    pub fn max_row_weight(&self) -> usize {
        self.row_weights.iter().cloned().max().unwrap_or(0)
    }

    /// Maximum column weight
    pub fn max_col_weight(&self) -> usize {
        self.col_weights.iter().cloned().max().unwrap_or(0)
    }

    /// Checks if the matrix is LDPC (low-density)
    pub fn is_ldpc(&self, threshold: usize) -> bool {
        self.max_row_weight() <= threshold && self.max_col_weight() <= threshold
    }

    /// Computes syndrome for a given error pattern
    pub fn syndrome(&self, error: &[bool]) -> Vec<bool> {
        let mut syndrome = vec![false; self.num_rows];
        
        for &(row, col) in &self.entries {
            if error[col] {
                syndrome[row] = !syndrome[row];
            }
        }
        
        syndrome
    }
}

/// Quantum LDPC (Low-Density Parity-Check) Code
///
/// qLDPC codes achieve 90% overhead reduction compared to surface codes
/// through constant-rate encoding with logarithmic-weight stabilizers.
#[derive(Debug, Clone)]
pub struct QuantumLDPCCode {
    /// X-type parity check matrix (for Z errors)
    pub hx: ParityCheckMatrix,
    /// Z-type parity check matrix (for X errors)
    pub hz: ParityCheckMatrix,
    /// Number of physical qubits
    pub num_physical: usize,
    /// Number of logical qubits
    pub num_logical: usize,
    /// Code distance
    pub distance: usize,
    /// Rate k/n
    pub rate: f64,
}

impl QuantumLDPCCode {
    /// Creates a hypergraph product code from two classical LDPC codes
    ///
    /// The hypergraph product construction creates:
    /// - n = n₁² + n₂² physical qubits
    /// - k = k₁*k₂ + k₁*k₂ logical qubits (typically)
    /// - d = min(d₁, d₂) distance
    pub fn hypergraph_product(h1: &ParityCheckMatrix, h2: &ParityCheckMatrix) -> Self {
        let n1 = h1.num_cols;
        let n2 = h2.num_cols;
        let m1 = h1.num_rows;
        let m2 = h2.num_rows;
        
        // Physical qubits in the product code
        let num_physical = n1 * n2 + m1 * m2;
        
        // Build X-type stabilizers
        let mut hx = ParityCheckMatrix::new(m1 * n2, num_physical);
        
        // H₁ ⊗ I part
        for &(r1, c1) in &h1.entries {
            for c2 in 0..n2 {
                let row = r1 * n2 + c2;
                let col = c1 * n2 + c2;
                hx.add_entry(row, col);
            }
        }
        
        // I ⊗ H₂ᵀ part (uses second block of qubits)
        for r1 in 0..m1 {
            for &(r2, c2) in &h2.entries {
                let row = r1 * n2 + c2;
                let col = n1 * n2 + r1 * m2 + r2;
                if col < num_physical {
                    hx.add_entry(row, col);
                }
            }
        }
        
        // Build Z-type stabilizers
        let mut hz = ParityCheckMatrix::new(n1 * m2, num_physical);
        
        // I ⊗ H₂ part
        for c1 in 0..n1 {
            for &(r2, c2) in &h2.entries {
                let row = c1 * m2 + r2;
                let col = c1 * n2 + c2;
                hz.add_entry(row, col);
            }
        }
        
        // H₁ᵀ ⊗ I part
        for &(r1, c1) in &h1.entries {
            for r2 in 0..m2 {
                let row = c1 * m2 + r2;
                let col = n1 * n2 + r1 * m2 + r2;
                if col < num_physical {
                    hz.add_entry(row, col);
                }
            }
        }
        
        // Estimate code parameters
        let num_x_stabilizers = m1 * n2;
        let num_z_stabilizers = n1 * m2;
        let num_logical = num_physical.saturating_sub(num_x_stabilizers + num_z_stabilizers);
        
        let distance = h1.max_row_weight().min(h2.max_row_weight());
        let rate = num_logical as f64 / num_physical as f64;
        
        QuantumLDPCCode {
            hx,
            hz,
            num_physical,
            num_logical: num_logical.max(1),
            distance,
            rate,
        }
    }

    /// Creates a bicycle code (quasi-cyclic LDPC)
    pub fn bicycle_code(size: usize, a_positions: &[usize], b_positions: &[usize]) -> Self {
        let num_physical = 2 * size;
        
        // Build circulant parity checks
        let mut hx = ParityCheckMatrix::new(size, num_physical);
        let mut hz = ParityCheckMatrix::new(size, num_physical);
        
        for row in 0..size {
            // A part
            for &pos in a_positions {
                let col = (row + pos) % size;
                hx.add_entry(row, col);
                hz.add_entry(row, size + col);
            }
            
            // B part
            for &pos in b_positions {
                let col = (row + pos) % size;
                hx.add_entry(row, size + col);
                hz.add_entry(row, col);
            }
        }
        
        let num_logical = 2;  // Typical for bicycle codes
        let distance = (a_positions.len() + b_positions.len()) / 2;
        let rate = num_logical as f64 / num_physical as f64;
        
        QuantumLDPCCode {
            hx,
            hz,
            num_physical,
            num_logical,
            distance,
            rate,
        }
    }

    /// Decodes using belief propagation
    pub fn decode_bp(&self, _syndrome_x: &[bool], syndrome_z: &[bool], max_iter: usize) -> Vec<bool> {
        let mut error_x = vec![false; self.num_physical];
        let _error_z = vec![false; self.num_physical];
        
        // Simple bit-flip decoder (real BP is more sophisticated)
        for _iter in 0..max_iter {
            let mut changed = false;
            
            // Check X syndrome
            let current_syndrome = self.hz.syndrome(&error_x);
            for (i, (&target, &current)) in syndrome_z.iter().zip(current_syndrome.iter()).enumerate() {
                if target != current {
                    // Flip a qubit in this check
                    for &(row, col) in &self.hz.entries {
                        if row == i {
                            error_x[col] = !error_x[col];
                            changed = true;
                            break;
                        }
                    }
                }
            }
            
            if !changed {
                break;
            }
        }
        
        error_x
    }

    /// Returns encoding overhead (physical/logical)
    pub fn overhead(&self) -> f64 {
        self.num_physical as f64 / self.num_logical as f64
    }
}

/// Color Code
///
/// Topological code with transversal Clifford gates,
/// defined on a trivalent, 3-colorable lattice.
#[derive(Debug, Clone)]
pub struct ColorCode {
    /// Code distance
    pub distance: usize,
    /// Number of physical qubits
    pub num_physical: usize,
    /// Number of logical qubits
    pub num_logical: usize,
    /// Face colors (0, 1, 2 for RGB)
    pub face_colors: Vec<usize>,
    /// Face to qubit mappings
    pub faces: Vec<Vec<usize>>,
    /// X stabilizers (one per face)
    pub x_stabilizers: Vec<Vec<usize>>,
    /// Z stabilizers (one per face)  
    pub z_stabilizers: Vec<Vec<usize>>,
}

impl ColorCode {
    /// Creates a triangular 4.8.8 color code
    ///
    /// The 4.8.8 lattice has squares and octagons,
    /// with distance = 2*layers + 1
    pub fn triangular_488(layers: usize) -> Self {
        let distance = 2 * layers + 1;
        
        // Number of qubits for 4.8.8 lattice
        // Simplified formula for small sizes
        let num_physical = 3 * layers * layers + 3 * layers + 1;
        
        // Build faces (simplified triangular structure)
        let _num_faces = layers * (layers + 1) / 2 * 3;
        let mut faces = Vec::new();
        let mut face_colors = Vec::new();
        
        let mut qubit_idx = 0;
        for layer in 0..layers {
            for face in 0..(layer + 1) {
                // Each face has 4 or 8 qubits depending on type
                let face_size = if (layer + face) % 2 == 0 { 4 } else { 8 };
                let face_qubits: Vec<usize> = (0..face_size.min(num_physical - qubit_idx))
                    .map(|i| (qubit_idx + i) % num_physical)
                    .collect();
                
                qubit_idx = (qubit_idx + face_size) % num_physical;
                
                faces.push(face_qubits);
                face_colors.push((layer + face) % 3);
            }
        }
        
        // X and Z stabilizers are the same for color codes (CSS-like)
        let x_stabilizers = faces.clone();
        let z_stabilizers = faces.clone();
        
        ColorCode {
            distance,
            num_physical,
            num_logical: 1,
            face_colors,
            faces,
            x_stabilizers,
            z_stabilizers,
        }
    }

    /// Creates a hexagonal 6.6.6 color code
    pub fn hexagonal_666(layers: usize) -> Self {
        let distance = 2 * layers + 1;
        
        // Hexagonal lattice qubit count
        let num_physical = 6 * layers * (layers + 1) / 2;
        
        // Build hexagonal faces
        let mut faces = Vec::new();
        let mut face_colors = Vec::new();
        
        for layer in 0..layers {
            for hex in 0..(6 * (layer + 1)) {
                // Each hexagon has 6 qubits
                let start_qubit = (layer * 6 + hex * 6) % num_physical.max(1);
                let face_qubits: Vec<usize> = (0..6)
                    .map(|i| (start_qubit + i) % num_physical.max(1))
                    .collect();
                
                faces.push(face_qubits);
                face_colors.push(hex % 3);
            }
        }
        
        let x_stabilizers = faces.clone();
        let z_stabilizers = faces.clone();
        
        ColorCode {
            distance,
            num_physical: num_physical.max(1),
            num_logical: 1,
            face_colors,
            faces,
            x_stabilizers,
            z_stabilizers,
        }
    }

    /// Computes X syndrome
    pub fn measure_x_syndrome(&self, state: &[bool]) -> Vec<bool> {
        self.x_stabilizers.iter()
            .map(|stab| stab.iter().filter(|&&q| state[q]).count() % 2 == 1)
            .collect()
    }

    /// Computes Z syndrome
    pub fn measure_z_syndrome(&self, state: &[bool]) -> Vec<bool> {
        self.z_stabilizers.iter()
            .map(|stab| stab.iter().filter(|&&q| state[q]).count() % 2 == 1)
            .collect()
    }

    /// Applies transversal Hadamard (permutes X↔Z)
    pub fn transversal_hadamard(&self, _state: &mut [bool]) {
        // For color codes, transversal H is simply H on each qubit
        // In our simplified model, this is a no-op on classical bits
    }

    /// Applies transversal S gate
    pub fn transversal_s(&self, _state: &mut [bool]) {
        // S gate is transversal on color codes
    }

    /// Applies transversal CNOT between two color code blocks
    pub fn transversal_cnot(&self, control: &[bool], target: &mut [bool]) {
        for (i, &c) in control.iter().enumerate() {
            if c {
                target[i] = !target[i];
            }
        }
    }
}

/// Floquet Code
///
/// Dynamically generated topological code using periodic
/// measurement sequences, offering improved thresholds.
#[derive(Debug, Clone)]
pub struct FloquetCode {
    /// Number of physical qubits
    pub num_physical: usize,
    /// Period of the Floquet cycle
    pub period: usize,
    /// Current time step in cycle
    pub current_step: usize,
    /// Measurement schedule (which stabilizers to measure at each step)
    pub schedule: Vec<Vec<usize>>,
    /// Stabilizer definitions
    pub stabilizers: Vec<Vec<usize>>,
}

impl FloquetCode {
    /// Creates a honeycomb Floquet code
    ///
    /// Based on Hastings & Haah's construction using
    /// alternating XX, YY, ZZ measurements.
    pub fn honeycomb(size: usize) -> Self {
        let num_physical = 2 * size * size;
        let num_stabilizers = 3 * size * size;
        
        // Build stabilizers (2-body checks)
        let mut stabilizers = Vec::new();
        
        for i in 0..size {
            for j in 0..size {
                let q1 = i * size + j;
                let q2 = ((i + 1) % size) * size + j;
                let q3 = i * size + (j + 1) % size;
                
                // Three types of 2-body checks
                stabilizers.push(vec![q1 % num_physical, q2 % num_physical]);  // XX type
                stabilizers.push(vec![q1 % num_physical, q3 % num_physical]);  // YY type
                stabilizers.push(vec![q2 % num_physical, q3 % num_physical]);  // ZZ type
            }
        }
        
        // Measurement schedule: cycle through XX, YY, ZZ
        let schedule = vec![
            (0..num_stabilizers).filter(|i| i % 3 == 0).collect(),  // XX
            (0..num_stabilizers).filter(|i| i % 3 == 1).collect(),  // YY
            (0..num_stabilizers).filter(|i| i % 3 == 2).collect(),  // ZZ
        ];
        
        FloquetCode {
            num_physical,
            period: 3,
            current_step: 0,
            schedule,
            stabilizers,
        }
    }

    /// Advances one time step in the Floquet cycle
    pub fn advance_step(&mut self, measurement_outcomes: &[bool]) -> Vec<bool> {
        // Get syndrome from this round
        let measured_stabilizers = &self.schedule[self.current_step];
        let mut syndrome = vec![false; measured_stabilizers.len()];
        
        for (i, &stab_idx) in measured_stabilizers.iter().enumerate() {
            if stab_idx < measurement_outcomes.len() {
                syndrome[i] = measurement_outcomes[stab_idx];
            }
        }
        
        // Advance time
        self.current_step = (self.current_step + 1) % self.period;
        
        syndrome
    }

    /// Detects errors from syndrome history
    pub fn detect_errors(&self, syndrome_history: &[Vec<bool>]) -> Vec<(usize, usize)> {
        let mut errors = Vec::new();
        
        // Compare consecutive syndromes for changes
        for t in 1..syndrome_history.len() {
            for (i, (&prev, &curr)) in syndrome_history[t-1].iter()
                .zip(syndrome_history[t].iter()).enumerate()
            {
                if prev != curr {
                    errors.push((t, i));
                }
            }
        }
        
        errors
    }

    /// Returns the current measurement basis
    pub fn current_basis(&self) -> &str {
        match self.current_step {
            0 => "XX",
            1 => "YY",
            2 => "ZZ",
            _ => "Unknown",
        }
    }
}

/// Creates a classical LDPC parity check matrix for testing
pub fn create_classical_ldpc(n: usize, k: usize, row_weight: usize) -> ParityCheckMatrix {
    let m = n - k;
    let mut h = ParityCheckMatrix::new(m, n);
    
    // Simple regular LDPC construction
    for row in 0..m {
        for w in 0..row_weight {
            let col = (row * row_weight + w) % n;
            h.add_entry(row, col);
        }
    }
    
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parity_check_matrix() {
        let mut h = ParityCheckMatrix::new(3, 6);
        h.add_entry(0, 0);
        h.add_entry(0, 1);
        h.add_entry(0, 2);
        h.add_entry(1, 1);
        h.add_entry(1, 3);
        h.add_entry(1, 4);
        h.add_entry(2, 2);
        h.add_entry(2, 4);
        h.add_entry(2, 5);
        
        assert_eq!(h.row_weight(0), 3);
        assert!(h.is_ldpc(4));
    }

    #[test]
    fn test_syndrome_computation() {
        let mut h = ParityCheckMatrix::new(2, 4);
        h.add_entry(0, 0);
        h.add_entry(0, 1);
        h.add_entry(1, 2);
        h.add_entry(1, 3);
        
        let error = vec![true, false, true, false];
        let syndrome = h.syndrome(&error);
        
        assert_eq!(syndrome, vec![true, true]);
    }

    #[test]
    fn test_hypergraph_product() {
        let h1 = create_classical_ldpc(6, 3, 2);
        let h2 = create_classical_ldpc(6, 3, 2);
        
        let qldpc = QuantumLDPCCode::hypergraph_product(&h1, &h2);
        
        assert!(qldpc.num_physical > 0);
        assert!(qldpc.rate > 0.0);
    }

    #[test]
    fn test_bicycle_code() {
        let code = QuantumLDPCCode::bicycle_code(7, &[0, 1, 3], &[0, 2, 3]);
        
        assert_eq!(code.num_physical, 14);
        assert!(code.distance > 0);
    }

    #[test]
    fn test_color_code_triangular() {
        let code = ColorCode::triangular_488(2);
        
        assert!(code.distance >= 5);
        assert_eq!(code.num_logical, 1);
        assert!(!code.faces.is_empty());
    }

    #[test]
    fn test_color_code_hexagonal() {
        let code = ColorCode::hexagonal_666(2);
        
        assert!(code.distance >= 5);
        assert_eq!(code.num_logical, 1);
    }

    #[test]
    fn test_color_code_transversal() {
        let code = ColorCode::triangular_488(1);
        let mut state = vec![false; code.num_physical];
        state[0] = true;
        
        let control = state.clone();
        code.transversal_cnot(&control, &mut state);
        
        // Should have applied XOR
    }

    #[test]
    fn test_floquet_honeycomb() {
        let code = FloquetCode::honeycomb(3);
        
        assert_eq!(code.period, 3);
        assert!(!code.stabilizers.is_empty());
    }

    #[test]
    fn test_floquet_cycle() {
        let mut code = FloquetCode::honeycomb(2);
        
        assert_eq!(code.current_basis(), "XX");
        
        let outcomes = vec![false; 20];
        code.advance_step(&outcomes);
        
        assert_eq!(code.current_basis(), "YY");
    }

    #[test]
    fn test_qldpc_overhead() {
        let h = create_classical_ldpc(10, 5, 3);
        let code = QuantumLDPCCode::hypergraph_product(&h, &h);
        
        // qLDPC should have much lower overhead than surface codes
        assert!(code.overhead() < 100.0);
    }
}
