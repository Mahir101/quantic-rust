//! Circuit Optimization Algorithms
//!
//! This module provides quantum circuit optimization techniques:
//! - Gate cancellation (removing consecutive inverse gates)
//! - Gate merging (combining compatible rotations)
//! - Commutation analysis
//! - T-count optimization
//! - CNOT minimization
//! - Template matching
//!
//! ## 🎯 Why is this used?
//! Synthesis algorithms often produce redundant or non-local gate sequences. This 
//! module is used to "clean" those circuits, reducing the number of 1-qubit and 
//! 2-qubit gates. This reduction is vital for reducing decoherence-related 
//! failures and improving the overall success rate of an algorithm execution.
//!
//! ## ⚙️ How it works?
//! - **Peephole Optimization**: Scans the circuit with a small window to find 
//!   sequences that match known identities (e.g., $H \cdot H = I$).
//! - **Commutation Analysis**: Tracks which gates can commute past one another 
//!   to enable distant gate cancellations (e.g., $Z$ moving past $CNOT$ control).
//! - **Rotation Merging**: Combines multiple rotations around the same axis 
//!   (e.g., $RZ(\theta_1) \cdot RZ(\theta_2) \rightarrow RZ(\theta_1 + \theta_2)$).
//! - **Template Matching**: Replaces specific sub-circuits with more efficient 
//!   equivalent versions from a pre-computed library.
//!
//! ## 📍 Where to apply this?
//! - **Post-Synthesis**: Immediately after generating a circuit from a matrix 
//!   or high-level routine (QFT, Arithmetic).
//! - **Transpilation**: During the mapping of a logical circuit to a physical 
//!   gate set.
//!
//! ## 📊 Code Behavior
//! - **Performance**: $O(G^2)$ in the worst case for commutation-based global 
//!   cancellation, but $O(G)$ for simple local peephole passes.
//! - **Side Effects**: May change the qubit dependency graph by reordering gates.
//! - **Precision**: Merged rotations use sum-of-angles which maintains $O(1)$ 
//!   relative precision error across merges.

use std::f64::consts::PI;
use crate::gates::core::Gate;

// ============================================================================
// GATE CANCELLATION
// ============================================================================

/// Remove consecutive inverse gates from a circuit
/// 
/// Identifies and removes pairs like (H, H), (X, X), (CX, CX), (T, Tdg), etc.
pub fn cancel_inverse_gates(circuit: &[Gate]) -> Vec<Gate> {
    let mut result = Vec::new();
    let mut i = 0;
    
    while i < circuit.len() {
        if i + 1 < circuit.len() && are_inverse_gates(&circuit[i], &circuit[i + 1]) {
            // Skip both gates
            i += 2;
        } else {
            result.push(circuit[i].clone());
            i += 1;
        }
    }
    
    // Recursively apply until no more cancellations
    if result.len() < circuit.len() {
        cancel_inverse_gates(&result)
    } else {
        result
    }
}

/// Check if two gates are inverses of each other
fn are_inverse_gates(g1: &Gate, g2: &Gate) -> bool {
    match (g1, g2) {
        // Self-inverse gates
        (Gate::X(q1), Gate::X(q2)) => q1 == q2,
        (Gate::Y(q1), Gate::Y(q2)) => q1 == q2,
        (Gate::Z(q1), Gate::Z(q2)) => q1 == q2,
        (Gate::H(q1), Gate::H(q2)) => q1 == q2,
        (Gate::CX(c1, t1), Gate::CX(c2, t2)) => c1 == c2 && t1 == t2,
        (Gate::CZ(c1, t1), Gate::CZ(c2, t2)) => c1 == c2 && t1 == t2,
        (Gate::SWAP(a1, b1), Gate::SWAP(a2, b2)) => 
            (a1 == a2 && b1 == b2) || (a1 == b2 && b1 == a2),
        
        // S and Sdg
        (Gate::S(q1), Gate::Sdg(q2)) => q1 == q2,
        (Gate::Sdg(q1), Gate::S(q2)) => q1 == q2,
        
        // T and Tdg
        (Gate::T(q1), Gate::Tdg(q2)) => q1 == q2,
        (Gate::Tdg(q1), Gate::T(q2)) => q1 == q2,
        
        // Rotation gates with opposite angles
        (Gate::RX(q1, theta1), Gate::RX(q2, theta2)) => 
            q1 == q2 && (theta1 + theta2).abs() < 1e-10,
        (Gate::RY(q1, theta1), Gate::RY(q2, theta2)) => 
            q1 == q2 && (theta1 + theta2).abs() < 1e-10,
        (Gate::RZ(q1, theta1), Gate::RZ(q2, theta2)) => 
            q1 == q2 && (theta1 + theta2).abs() < 1e-10,
        (Gate::P(q1, phi1), Gate::P(q2, phi2)) => 
            q1 == q2 && (phi1 + phi2).abs() < 1e-10,
        
        _ => false,
    }
}

// ============================================================================
// GATE MERGING
// ============================================================================

/// Merge consecutive rotation gates on the same qubit
/// 
/// Combines RZ(θ₁)RZ(θ₂) → RZ(θ₁+θ₂), etc.
pub fn merge_rotations(circuit: &[Gate]) -> Vec<Gate> {
    let mut result = Vec::new();
    let mut i = 0;
    
    while i < circuit.len() {
        let merged = try_merge_rotation(&circuit[i..]);
        if merged.0 > 1 {
            // Successfully merged multiple gates
            result.push(merged.1);
            i += merged.0;
        } else {
            result.push(circuit[i].clone());
            i += 1;
        }
    }
    
    result
}

/// Try to merge consecutive rotation gates
/// Returns (number of gates merged, resulting gate)
fn try_merge_rotation(gates: &[Gate]) -> (usize, Gate) {
    if gates.is_empty() {
        return (0, Gate::H(0)); // Placeholder
    }
    
    match &gates[0] {
        Gate::RZ(q, theta) => {
            let mut total_angle = *theta;
            let mut count = 1;
            
            for gate in &gates[1..] {
                match gate {
                    Gate::RZ(q2, theta2) if q2 == q => {
                        total_angle += theta2;
                        count += 1;
                    }
                    _ => break,
                }
            }
            
            // Remove if angle is multiple of 2π
            if (total_angle % (2.0 * PI)).abs() < 1e-10 {
                (count, Gate::RZ(*q, 0.0)) // Will be removed in cleanup
            } else {
                (count, Gate::RZ(*q, total_angle))
            }
        }
        Gate::RX(q, theta) => {
            let mut total_angle = *theta;
            let mut count = 1;
            
            for gate in &gates[1..] {
                match gate {
                    Gate::RX(q2, theta2) if q2 == q => {
                        total_angle += theta2;
                        count += 1;
                    }
                    _ => break,
                }
            }
            
            (count, Gate::RX(*q, total_angle))
        }
        Gate::RY(q, theta) => {
            let mut total_angle = *theta;
            let mut count = 1;
            
            for gate in &gates[1..] {
                match gate {
                    Gate::RY(q2, theta2) if q2 == q => {
                        total_angle += theta2;
                        count += 1;
                    }
                    _ => break,
                }
            }
            
            (count, Gate::RY(*q, total_angle))
        }
        Gate::P(q, phi) => {
            let mut total_angle = *phi;
            let mut count = 1;
            
            for gate in &gates[1..] {
                match gate {
                    Gate::P(q2, phi2) if q2 == q => {
                        total_angle += phi2;
                        count += 1;
                    }
                    _ => break,
                }
            }
            
            (count, Gate::P(*q, total_angle))
        }
        _ => (1, gates[0].clone()),
    }
}

// ============================================================================
// COMMUTATION ANALYSIS
// ============================================================================

/// Check if two gates commute
pub fn gates_commute(g1: &Gate, g2: &Gate) -> bool {
    // Gates on different qubits always commute
    let q1: std::collections::HashSet<_> = g1.qubits().into_iter().collect();
    let q2: std::collections::HashSet<_> = g2.qubits().into_iter().collect();
    
    if q1.is_disjoint(&q2) {
        return true;
    }
    
    // Same-qubit commutation rules
    match (g1, g2) {
        // Diagonal gates commute with each other
        (Gate::RZ(_, _), Gate::RZ(_, _)) => true,
        (Gate::P(_, _), Gate::P(_, _)) => true,
        (Gate::Z(_), Gate::Z(_)) => true,
        (Gate::S(_), Gate::S(_)) => true,
        (Gate::T(_), Gate::T(_)) => true,
        (Gate::CZ(_, _), Gate::CZ(_, _)) => true,
        
        // RZ commutes with Z-type gates
        (Gate::RZ(_, _), Gate::Z(_)) | (Gate::Z(_), Gate::RZ(_, _)) => true,
        (Gate::RZ(_, _), Gate::S(_)) | (Gate::S(_), Gate::RZ(_, _)) => true,
        (Gate::RZ(_, _), Gate::T(_)) | (Gate::T(_), Gate::RZ(_, _)) => true,
        
        // Control commutes with CZ target
        (Gate::RZ(q, _), Gate::CZ(_, t)) | (Gate::CZ(_, t), Gate::RZ(q, _)) if q == t => true,
        
        // CNOT commutes with Z on control, X on target
        (Gate::Z(q), Gate::CX(c, _)) | (Gate::CX(c, _), Gate::Z(q)) if q == c => true,
        (Gate::X(q), Gate::CX(_, t)) | (Gate::CX(_, t), Gate::X(q)) if q == t => true,
        
        _ => false,
    }
}

/// Reorder gates using commutation rules to group cancellable pairs
pub fn commute_and_cancel(circuit: &[Gate]) -> Vec<Gate> {
    let mut result: Vec<Gate> = circuit.to_vec();
    let mut changed = true;
    
    while changed {
        changed = false;
        
        for i in 0..result.len().saturating_sub(1) {
            // Try to move gates closer if they might cancel
            if can_move_closer(&result, i) {
                // Swap adjacent gates if they commute and moving helps
                if i + 2 < result.len() && 
                   gates_commute(&result[i + 1], &result[i + 2]) &&
                   are_inverse_gates(&result[i], &result[i + 2]) {
                    result.swap(i + 1, i + 2);
                    changed = true;
                }
            }
        }
        
        // Apply cancellation
        let new_result = cancel_inverse_gates(&result);
        if new_result.len() < result.len() {
            result = new_result;
            changed = true;
        }
    }
    
    result
}

fn can_move_closer(circuit: &[Gate], idx: usize) -> bool {
    if idx + 2 >= circuit.len() {
        return false;
    }
    // Check if there's a potential cancellation opportunity
    are_inverse_gates(&circuit[idx], &circuit[idx + 2])
}

// ============================================================================
// T-COUNT OPTIMIZATION
// ============================================================================

/// Count T and Tdg gates in a circuit
pub fn t_count(circuit: &[Gate]) -> usize {
    circuit.iter().filter(|g| matches!(g, Gate::T(_) | Gate::Tdg(_))).count()
}

/// Count T-depth (maximum T gates in any parallel layer)
pub fn t_depth(circuit: &[Gate]) -> usize {
    let mut depths: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    let mut max_depth = 0;
    
    for gate in circuit {
        match gate {
            Gate::T(q) | Gate::Tdg(q) => {
                let new_depth = depths.get(q).unwrap_or(&0) + 1;
                depths.insert(*q, new_depth);
                max_depth = max_depth.max(new_depth);
            }
            _ => {
                // Non-T gates don't affect T-depth
            }
        }
    }
    
    max_depth
}

/// Optimize T-count using Reed-Muller decomposition identities
/// 
/// Common identities:
/// - T T T T = Z
/// - T T = S
/// - Tdg Tdg = Sdg
pub fn optimize_t_gates(circuit: &[Gate]) -> Vec<Gate> {
    let mut result = Vec::new();
    let mut t_count_per_qubit: std::collections::HashMap<usize, i32> = 
        std::collections::HashMap::new();
    
    for gate in circuit {
        match gate {
            Gate::T(q) => {
                let count = t_count_per_qubit.entry(*q).or_insert(0);
                *count += 1;
                
                // Check for pattern matches
                *count = match *count {
                    4 => {
                        result.push(Gate::Z(*q));
                        0
                    }
                    2 => {
                        result.push(Gate::S(*q));
                        0
                    }
                    _ => *count,
                };
            }
            Gate::Tdg(q) => {
                let count = t_count_per_qubit.entry(*q).or_insert(0);
                *count -= 1;
                
                *count = match *count {
                    -4 => {
                        result.push(Gate::Z(*q));
                        0
                    }
                    -2 => {
                        result.push(Gate::Sdg(*q));
                        0
                    }
                    _ => *count,
                };
            }
            _ => {
                // Flush any pending T gates before non-commuting gate
                flush_t_gates(&mut result, &mut t_count_per_qubit, &gate.qubits());
                result.push(gate.clone());
            }
        }
    }
    
    // Flush remaining T gates
    flush_all_t_gates(&mut result, &t_count_per_qubit);
    
    result
}

fn flush_t_gates(
    result: &mut Vec<Gate>,
    t_counts: &mut std::collections::HashMap<usize, i32>,
    qubits: &[usize],
) {
    for q in qubits {
        if let Some(count) = t_counts.remove(q) {
            add_t_gates(result, *q, count);
        }
    }
}

fn flush_all_t_gates(
    result: &mut Vec<Gate>,
    t_counts: &std::collections::HashMap<usize, i32>,
) {
    for (&q, &count) in t_counts {
        add_t_gates(result, q, count);
    }
}

fn add_t_gates(result: &mut Vec<Gate>, q: usize, count: i32) {
    let count = count % 8;
    match count {
        0 => {}
        1 => result.push(Gate::T(q)),
        2 => result.push(Gate::S(q)),
        3 => {
            result.push(Gate::S(q));
            result.push(Gate::T(q));
        }
        4 | -4 => result.push(Gate::Z(q)),
        -1 | 7 => result.push(Gate::Tdg(q)),
        -2 | 6 => result.push(Gate::Sdg(q)),
        -3 | 5 => {
            result.push(Gate::Sdg(q));
            result.push(Gate::Tdg(q));
        }
        _ => {
            for _ in 0..count.abs() {
                if count > 0 {
                    result.push(Gate::T(q));
                } else {
                    result.push(Gate::Tdg(q));
                }
            }
        }
    }
}

// ============================================================================
// CNOT MINIMIZATION
// ============================================================================

/// Count CNOT gates in a circuit
pub fn cnot_count(circuit: &[Gate]) -> usize {
    circuit.iter().filter(|g| matches!(g, Gate::CX(_, _))).count()
}

/// Compute CNOT depth
pub fn cnot_depth(circuit: &[Gate]) -> usize {
    let mut depths: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    let mut max_depth = 0;
    
    for gate in circuit {
        if let Gate::CX(c, t) = gate {
            let c_depth = depths.get(c).unwrap_or(&0);
            let t_depth = depths.get(t).unwrap_or(&0);
            let new_depth = c_depth.max(t_depth) + 1;
            depths.insert(*c, new_depth);
            depths.insert(*t, new_depth);
            max_depth = max_depth.max(new_depth);
        }
    }
    
    max_depth
}

/// Apply CNOT cancellation rules
/// 
/// - CX CX = I
/// - CX(a,b) CX(b,c) CX(a,b) = CX(a,c) CX(b,c)
pub fn optimize_cnot_gates(circuit: &[Gate]) -> Vec<Gate> {
    let mut result = circuit.to_vec();
    let mut changed = true;
    
    while changed {
        changed = false;
        
        // Rule 1: Cancel adjacent CNOTs
        let new_result = cancel_inverse_gates(&result);
        if new_result.len() < result.len() {
            result = new_result;
            changed = true;
            continue;
        }
        
        // Rule 2: CX(a,b) CX(b,c) CX(a,b) = CX(a,c) CX(b,c)
        for i in 0..result.len().saturating_sub(2) {
            let pattern_match = if let (Gate::CX(a1, b1), Gate::CX(b2, c), Gate::CX(a3, b3)) = 
                (&result[i], &result[i+1], &result[i+2]) 
            {
                if a1 == a3 && b1 == b3 && b1 == b2 && c != a1 {
                    Some((*a1, *b1, *c))
                } else {
                    None
                }
            } else {
                None
            };

            if let Some((a, b, c)) = pattern_match {
                result[i] = Gate::CX(a, c);
                result[i+1] = Gate::CX(b, c);
                result.remove(i+2);
                changed = true;
                break;
            }
        }
    }
    
    result
}

// ============================================================================
// TEMPLATE MATCHING
// ============================================================================

/// A circuit template for pattern-based optimization
#[derive(Clone)]
pub struct CircuitTemplate {
    pub pattern: Vec<Gate>,
    pub replacement: Vec<Gate>,
}

/// Apply template-based optimization
pub fn apply_templates(circuit: &[Gate], templates: &[CircuitTemplate]) -> Vec<Gate> {
    let mut result = circuit.to_vec();
    
    for template in templates {
        result = apply_template(&result, template);
    }
    
    result
}

fn apply_template(circuit: &[Gate], template: &CircuitTemplate) -> Vec<Gate> {
    let pattern_len = template.pattern.len();
    if circuit.len() < pattern_len {
        return circuit.to_vec();
    }
    
    let mut result = Vec::new();
    let mut i = 0;
    
    while i < circuit.len() {
        if i + pattern_len <= circuit.len() && matches_template(&circuit[i..i+pattern_len], &template.pattern) {
            result.extend(template.replacement.clone());
            i += pattern_len;
        } else {
            result.push(circuit[i].clone());
            i += 1;
        }
    }
    
    result
}

fn matches_template(circuit_slice: &[Gate], pattern: &[Gate]) -> bool {
    if circuit_slice.len() != pattern.len() {
        return false;
    }
    
    // Build qubit mapping from pattern to circuit
    let mut qubit_map: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    
    for (c_gate, p_gate) in circuit_slice.iter().zip(pattern.iter()) {
        if !gate_matches(c_gate, p_gate, &mut qubit_map) {
            return false;
        }
    }
    
    true
}

fn gate_matches(
    circuit_gate: &Gate,
    pattern_gate: &Gate,
    qubit_map: &mut std::collections::HashMap<usize, usize>,
) -> bool {
    let c_qubits = circuit_gate.qubits();
    let p_qubits = pattern_gate.qubits();
    
    if c_qubits.len() != p_qubits.len() {
        return false;
    }
    
    // Check qubit mapping consistency
    for (&c_q, &p_q) in c_qubits.iter().zip(p_qubits.iter()) {
        if let Some(&mapped) = qubit_map.get(&p_q) {
            if mapped != c_q {
                return false;
            }
        } else {
            qubit_map.insert(p_q, c_q);
        }
    }
    
    // Check gate type matches
    std::mem::discriminant(circuit_gate) == std::mem::discriminant(pattern_gate)
}

// ============================================================================
// PEEPHOLE OPTIMIZATION
// ============================================================================

/// Apply peephole optimization (local optimizations on small windows)
pub fn peephole_optimize(circuit: &[Gate], window_size: usize) -> Vec<Gate> {
    let mut result = circuit.to_vec();
    let mut changed = true;
    
    while changed {
        changed = false;
        
        let mut i = 0;
        while i + window_size <= result.len() {
            let window = &result[i..i+window_size];
            
            if let Some(optimized) = optimize_window(window) {
                // Replace window with optimized version
                let mut new_result = result[..i].to_vec();
                new_result.extend(optimized);
                new_result.extend(result[i+window_size..].to_vec());
                
                if new_result.len() < result.len() {
                    result = new_result;
                    changed = true;
                    break;
                }
            }
            i += 1;
        }
    }
    
    result
}

fn optimize_window(window: &[Gate]) -> Option<Vec<Gate>> {
    // Apply all optimization rules
    let optimized = cancel_inverse_gates(window);
    if optimized.len() < window.len() {
        return Some(optimized);
    }
    
    let optimized = merge_rotations(window);
    if optimized.len() < window.len() {
        return Some(optimized);
    }
    
    None
}

// ============================================================================
// FULL OPTIMIZATION PIPELINE
// ============================================================================

/// Apply full optimization pipeline
pub fn full_optimize(circuit: &[Gate]) -> Vec<Gate> {
    let mut result = circuit.to_vec();
    
    // Phase 1: Cancel inverse gates
    result = cancel_inverse_gates(&result);
    
    // Phase 2: Merge rotations
    result = merge_rotations(&result);
    
    // Phase 3: Commute and cancel
    result = commute_and_cancel(&result);
    
    // Phase 4: T-gate optimization
    result = optimize_t_gates(&result);
    
    // Phase 5: CNOT optimization
    result = optimize_cnot_gates(&result);
    
    // Phase 6: Peephole optimization
    result = peephole_optimize(&result, 4);
    
    // Final cleanup
    result = cancel_inverse_gates(&result);
    
    // Remove identity gates
    result.retain(|g| match g {
        Gate::RZ(_, theta) | Gate::RX(_, theta) | Gate::RY(_, theta) => theta.abs() > 1e-10,
        Gate::P(_, phi) => phi.abs() > 1e-10,
        _ => true,
    });
    
    result
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cancel_inverse() {
        let circuit = vec![Gate::H(0), Gate::H(0), Gate::X(1)];
        let result = cancel_inverse_gates(&circuit);
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Gate::X(1)));
    }

    #[test]
    fn test_merge_rotations() {
        let circuit = vec![
            Gate::RZ(0, PI / 4.0),
            Gate::RZ(0, PI / 4.0),
        ];
        let result = merge_rotations(&circuit);
        assert_eq!(result.len(), 1);
        if let Gate::RZ(_, theta) = result[0] {
            assert!((theta - PI / 2.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_gates_commute() {
        assert!(gates_commute(&Gate::RZ(0, 1.0), &Gate::RZ(0, 2.0)));
        assert!(gates_commute(&Gate::X(0), &Gate::Y(1)));
        assert!(!gates_commute(&Gate::X(0), &Gate::Y(0)));
    }

    #[test]
    fn test_t_count() {
        let circuit = vec![
            Gate::T(0), Gate::T(0), Gate::Tdg(1), Gate::H(0),
        ];
        assert_eq!(t_count(&circuit), 3);
    }

    #[test]
    fn test_cnot_optimization() {
        let circuit = vec![
            Gate::CX(0, 1), Gate::CX(0, 1),
        ];
        let result = optimize_cnot_gates(&circuit);
        assert!(result.is_empty());
    }
}
