//! # Lattice Cryptography Foundations
//!
//! ## 🎯 Purpose & Motivation
//!
//! This module provides the mathematical foundations for lattice-based cryptography,
//! which forms the basis of NIST's post-quantum cryptography standards (ML-KEM, ML-DSA).
//!
//! ## ⚙️ Core Concepts
//!
//! ### What is a Lattice?
//!
//! A **lattice** Λ in ℝⁿ is the set of all integer linear combinations of n
//! linearly independent vectors b₁, ..., bₙ:
//!
//! ```text
//! Λ = { Σᵢ zᵢbᵢ : zᵢ ∈ ℤ }
//! ```
//!
//! ### Hard Lattice Problems
//!
//! 1. **Shortest Vector Problem (SVP)**: Find the shortest non-zero vector in Λ
//! 2. **Closest Vector Problem (CVP)**: Find the closest lattice point to a target
//! 3. **Learning With Errors (LWE)**: Distinguish (A, As+e) from random
//!
//! These problems are believed to be hard even for quantum computers.
//!
//! ### Ring Structures for Efficiency
//!
//! - **Ring-LWE**: Uses polynomial ring R = ℤ[X]/(Xⁿ+1)
//! - **Module-LWE**: Uses Rₖ for configurable security levels
//!
//! ## 📊 Parameter Sets (ML-KEM/Kyber)
//!
//! | Parameter | ML-KEM-512 | ML-KEM-768 | ML-KEM-1024 |
//! |-----------|------------|------------|-------------|
//! | Security | Level 1 | Level 3 | Level 5 |
//! | n (ring dim) | 256 | 256 | 256 |
//! | k (module rank) | 2 | 3 | 4 |
//! | q (modulus) | 3329 | 3329 | 3329 |
//!
//! ## 📚 References
//!
//! - Regev, O. (2005). "On lattices, learning with errors, random linear codes..."
//! - Lyubashevsky et al. (2010). "On ideal lattices and learning with errors..."

/// The prime modulus q = 3329 used in ML-KEM (Kyber) and ML-DSA (Dilithium)
/// This is chosen such that q ≡ 1 (mod 256), enabling efficient NTT.
pub const MLKEM_Q: i32 = 3329;

/// Ring dimension n = 256, degree of the polynomial X^n + 1
pub const MLKEM_N: usize = 256;

/// Primitive 512th root of unity modulo q = 3329
/// This is used for Number Theoretic Transform (NTT)
pub const MLKEM_ZETA: i32 = 17;

/// Represents an element in the polynomial ring R_q = Z_q[X]/(X^n + 1)
///
/// Polynomials are stored as coefficient vectors in little-endian order:
/// p(X) = coeffs[0] + coeffs[1]*X + ... + coeffs[n-1]*X^(n-1)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Polynomial {
    /// Coefficient vector (length n = 256)
    pub coeffs: Vec<i32>,
}

impl Polynomial {
    /// Creates a zero polynomial
    pub fn zero() -> Self {
        Polynomial {
            coeffs: vec![0; MLKEM_N],
        }
    }

    /// Creates a polynomial from coefficients
    ///
    /// Coefficients are automatically reduced modulo q
    pub fn new(coeffs: Vec<i32>) -> Self {
        let mut p = Polynomial { coeffs };
        p.reduce();
        p
    }

    /// Creates a polynomial with random coefficients
    ///
    /// Uses a simple PRNG seeded with `seed` for reproducibility
    pub fn random(seed: u64) -> Self {
        let mut rng_state = seed;
        let coeffs: Vec<i32> = (0..MLKEM_N)
            .map(|_| {
                rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
                ((rng_state >> 33) % (MLKEM_Q as u64)) as i32
            })
            .collect();
        Polynomial { coeffs }
    }

    /// Creates a polynomial from centered binomial distribution
    ///
    /// This is used for generating error/secret polynomials in LWE.
    /// η (eta) determines the distribution width.
    pub fn sample_cbd(seed: u64, eta: usize) -> Self {
        let mut rng_state = seed;
        let mut coeffs = vec![0i32; MLKEM_N];
        
        for i in 0..MLKEM_N {
            let mut sum = 0i32;
            for _ in 0..(2 * eta) {
                rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
                if rng_state >> 63 == 1 {
                    sum += 1;
                }
            }
            // Center around zero
            coeffs[i] = (sum - eta as i32).rem_euclid(MLKEM_Q);
        }
        
        Polynomial { coeffs }
    }

    /// Reduces all coefficients modulo q
    pub fn reduce(&mut self) {
        for c in &mut self.coeffs {
            *c = c.rem_euclid(MLKEM_Q);
        }
    }

    /// Adds two polynomials
    pub fn add(&self, other: &Polynomial) -> Polynomial {
        let coeffs: Vec<i32> = self.coeffs.iter()
            .zip(other.coeffs.iter())
            .map(|(&a, &b)| (a + b).rem_euclid(MLKEM_Q))
            .collect();
        Polynomial { coeffs }
    }

    /// Subtracts two polynomials
    pub fn sub(&self, other: &Polynomial) -> Polynomial {
        let coeffs: Vec<i32> = self.coeffs.iter()
            .zip(other.coeffs.iter())
            .map(|(&a, &b)| (a - b).rem_euclid(MLKEM_Q))
            .collect();
        Polynomial { coeffs }
    }

    /// Multiplies two polynomials in R_q = Z_q[X]/(X^n + 1)
    ///
    /// This is the "schoolbook" method - O(n²).
    /// Production implementations use NTT for O(n log n).
    pub fn mul(&self, other: &Polynomial) -> Polynomial {
        let n = MLKEM_N;
        let mut result = vec![0i64; 2 * n - 1];
        
        // Standard polynomial multiplication
        for i in 0..n {
            for j in 0..n {
                result[i + j] += (self.coeffs[i] as i64) * (other.coeffs[j] as i64);
            }
        }
        
        // Reduce modulo X^n + 1: X^n ≡ -1
        let mut coeffs = vec![0i32; n];
        for i in 0..n {
            let high = if i + n < result.len() { result[i + n] } else { 0 };
            coeffs[i] = ((result[i] - high) % (MLKEM_Q as i64)) as i32;
            if coeffs[i] < 0 {
                coeffs[i] += MLKEM_Q;
            }
        }
        
        Polynomial { coeffs }
    }

    /// Scalar multiplication
    pub fn scalar_mul(&self, scalar: i32) -> Polynomial {
        let coeffs: Vec<i32> = self.coeffs.iter()
            .map(|&c| (c * scalar).rem_euclid(MLKEM_Q))
            .collect();
        Polynomial { coeffs }
    }

    /// Computes the infinity norm (max absolute coefficient)
    pub fn infinity_norm(&self) -> i32 {
        self.coeffs.iter()
            .map(|&c| {
                let c_centered = if c > MLKEM_Q / 2 { c - MLKEM_Q } else { c };
                c_centered.abs()
            })
            .max()
            .unwrap_or(0)
    }

    /// Encodes polynomial to bytes (for transmission)
    pub fn to_bytes(&self) -> Vec<u8> {
        // Simple encoding: 2 bytes per coefficient (12 bits needed for q=3329)
        let mut bytes = Vec::with_capacity(MLKEM_N * 2);
        for &c in &self.coeffs {
            bytes.push((c & 0xFF) as u8);
            bytes.push(((c >> 8) & 0xFF) as u8);
        }
        bytes
    }

    /// Decodes polynomial from bytes
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < MLKEM_N * 2 {
            return None;
        }
        
        let coeffs: Vec<i32> = (0..MLKEM_N)
            .map(|i| {
                let low = bytes[2 * i] as i32;
                let high = bytes[2 * i + 1] as i32;
                (low | (high << 8)) % MLKEM_Q
            })
            .collect();
        
        Some(Polynomial { coeffs })
    }
}

/// A vector of polynomials (module element)
///
/// Used in Module-LWE based schemes like ML-KEM and ML-DSA
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolynomialVector {
    pub polys: Vec<Polynomial>,
}

impl PolynomialVector {
    /// Creates a zero vector of k polynomials
    pub fn zero(k: usize) -> Self {
        PolynomialVector {
            polys: (0..k).map(|_| Polynomial::zero()).collect(),
        }
    }

    /// Creates a random vector
    pub fn random(k: usize, seed: u64) -> Self {
        PolynomialVector {
            polys: (0..k)
                .map(|i| Polynomial::random(seed.wrapping_add(i as u64)))
                .collect(),
        }
    }

    /// Sample from centered binomial distribution
    pub fn sample_cbd(k: usize, seed: u64, eta: usize) -> Self {
        PolynomialVector {
            polys: (0..k)
                .map(|i| Polynomial::sample_cbd(seed.wrapping_add(i as u64 * 1000), eta))
                .collect(),
        }
    }

    /// Vector addition
    pub fn add(&self, other: &PolynomialVector) -> PolynomialVector {
        PolynomialVector {
            polys: self.polys.iter()
                .zip(other.polys.iter())
                .map(|(a, b)| a.add(b))
                .collect(),
        }
    }

    /// Vector subtraction
    pub fn sub(&self, other: &PolynomialVector) -> PolynomialVector {
        PolynomialVector {
            polys: self.polys.iter()
                .zip(other.polys.iter())
                .map(|(a, b)| a.sub(b))
                .collect(),
        }
    }

    /// Inner product of two vectors (returns a polynomial)
    pub fn inner_product(&self, other: &PolynomialVector) -> Polynomial {
        let mut result = Polynomial::zero();
        for (a, b) in self.polys.iter().zip(other.polys.iter()) {
            result = result.add(&a.mul(b));
        }
        result
    }

    /// Dimension (number of polynomials)
    pub fn len(&self) -> usize {
        self.polys.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.polys.is_empty()
    }
}

/// A matrix of polynomials
///
/// Used for the public matrix A in Module-LWE
#[derive(Debug, Clone)]
pub struct PolynomialMatrix {
    pub rows: Vec<PolynomialVector>,
}

impl PolynomialMatrix {
    /// Creates a k×k zero matrix
    pub fn zero(k: usize) -> Self {
        PolynomialMatrix {
            rows: (0..k).map(|_| PolynomialVector::zero(k)).collect(),
        }
    }

    /// Creates a random k×k matrix
    pub fn random(k: usize, seed: u64) -> Self {
        PolynomialMatrix {
            rows: (0..k)
                .map(|i| PolynomialVector::random(k, seed.wrapping_add(i as u64 * 10000)))
                .collect(),
        }
    }

    /// Matrix-vector multiplication A * v
    pub fn mul_vector(&self, v: &PolynomialVector) -> PolynomialVector {
        PolynomialVector {
            polys: self.rows.iter()
                .map(|row| row.inner_product(v))
                .collect(),
        }
    }

    /// Transpose of the matrix
    pub fn transpose(&self) -> PolynomialMatrix {
        let k = self.rows.len();
        if k == 0 {
            return PolynomialMatrix { rows: vec![] };
        }
        
        let m = self.rows[0].len();
        PolynomialMatrix {
            rows: (0..m)
                .map(|j| PolynomialVector {
                    polys: (0..k)
                        .map(|i| self.rows[i].polys[j].clone())
                        .collect(),
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_polynomial_zero() {
        let p = Polynomial::zero();
        assert_eq!(p.coeffs.len(), MLKEM_N);
        assert!(p.coeffs.iter().all(|&c| c == 0));
    }

    #[test]
    fn test_polynomial_add() {
        let p1 = Polynomial::new(vec![1, 2, 3]);
        let p2 = Polynomial::new(vec![4, 5, 6]);
        let sum = p1.add(&p2);
        assert_eq!(sum.coeffs[0], 5);
        assert_eq!(sum.coeffs[1], 7);
        assert_eq!(sum.coeffs[2], 9);
    }

    #[test]
    fn test_polynomial_mul_mod() {
        // Test that X^n ≡ -1 in R_q
        let mut x_n: Vec<i32> = vec![0; MLKEM_N + 1];
        x_n[MLKEM_N] = 1;  // X^n
        
        let p = Polynomial::new(x_n);
        // X^n should reduce to -1 ≡ q-1
        assert_eq!(p.coeffs[0], MLKEM_Q - 1);
    }

    #[test]
    fn test_polynomial_serialization() {
        let p = Polynomial::random(12345);
        let bytes = p.to_bytes();
        let p2 = Polynomial::from_bytes(&bytes).unwrap();
        assert_eq!(p.coeffs, p2.coeffs);
    }

    #[test]
    fn test_vector_inner_product() {
        let v1 = PolynomialVector::random(3, 100);
        let v2 = PolynomialVector::random(3, 200);
        let ip = v1.inner_product(&v2);
        assert_eq!(ip.coeffs.len(), MLKEM_N);
    }

    #[test]
    fn test_matrix_vector_mul() {
        let a = PolynomialMatrix::random(2, 12345);
        let v = PolynomialVector::random(2, 67890);
        let result = a.mul_vector(&v);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_infinity_norm() {
        let p = Polynomial::new(vec![100, -50, 200]);
        let norm = p.infinity_norm();
        assert_eq!(norm, 200);
    }
}
