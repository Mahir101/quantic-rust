//! # Learning With Errors (LWE) - Foundation of Lattice Cryptography
//!
//! ## 🎯 Purpose & Motivation
//!
//! The **Learning With Errors (LWE)** problem is the security foundation for most
//! modern post-quantum cryptographic schemes, including ML-KEM (Kyber) and ML-DSA (Dilithium).
//! It was introduced by Oded Regev in 2005.
//!
//! ## ⚙️ The LWE Problem
//!
//! ### Definition
//!
//! Given:
//! - A random matrix A ∈ Z_q^(m×n)
//! - A secret vector s ∈ Z_q^n
//! - An error vector e ∈ Z_q^m with small entries (sampled from error distribution χ)
//! - The vector b = A·s + e (mod q)
//!
//! **Search-LWE**: Find s given (A, b)
//! **Decision-LWE**: Distinguish (A, b) from (A, u) where u is random
//!
//! ### Why is LWE Hard?
//!
//! 1. **Worst-case to Average-case Reduction**: LWE is as hard as worst-case
//!    lattice problems (approximate SVP, SIVP)
//! 2. **Quantum Resistance**: No known quantum algorithm solves LWE efficiently
//! 3. **Noise is Critical**: The error term e makes the problem hard
//!
//! ## 📊 Parameters
//!
//! | Parameter | Description | Typical Values |
//! |-----------|-------------|----------------|
//! | n | Secret dimension | 256-1024 |
//! | q | Modulus | 3329 (ML-KEM) |
//! | m | Number of samples | n to 2n |
//! | σ | Error standard deviation | Small (≈3) |
//!
//! ## 📍 Use Cases
//!
//! - **Key Exchange**: ML-KEM (Kyber)
//! - **Digital Signatures**: ML-DSA (Dilithium)  
//! - **Fully Homomorphic Encryption**: BGV, CKKS schemes
//! - **Identity-Based Encryption**
//!
//! ## 📚 References
//!
//! - Regev, O. (2005). "On lattices, learning with errors, random linear codes, and cryptography"
//! - Peikert, C. (2016). "A Decade of Lattice Cryptography"

use super::lattice::{MLKEM_Q, Polynomial, PolynomialVector, PolynomialMatrix};

/// LWE Error Distribution Types
#[derive(Debug, Clone, Copy)]
pub enum LweErrorDistribution {
    /// Centered Binomial Distribution with parameter η
    /// Used in Kyber for efficiency
    CenteredBinomial { eta: usize },
    /// Discrete Gaussian with standard deviation σ
    /// Provides better security proofs
    DiscreteGaussian { sigma: f64 },
    /// Uniform distribution over [-bound, bound]
    /// Simple but requires larger parameters
    Uniform { bound: i32 },
}

/// LWE Instance representing the public key and encrypted data
#[derive(Debug, Clone)]
pub struct LweInstance {
    /// Public matrix A (m × n)
    pub matrix_a: Vec<Vec<i32>>,
    /// Public vector b = A·s + e
    pub vector_b: Vec<i32>,
    /// Dimension n (secret vector size)
    pub n: usize,
    /// Number of samples m
    pub m: usize,
    /// Modulus q
    pub q: i32,
}

/// LWE Secret Key
#[derive(Debug, Clone)]
pub struct LweSecretKey {
    /// Secret vector s ∈ Z_q^n
    pub secret: Vec<i32>,
    /// Dimension
    pub n: usize,
    /// Modulus
    pub q: i32,
}

impl LweInstance {
    /// Generates a new LWE instance with the given parameters
    ///
    /// # Arguments
    /// * `n` - Secret dimension
    /// * `m` - Number of samples (LWE equations)
    /// * `q` - Modulus
    /// * `distribution` - Error distribution
    /// * `seed` - Random seed for reproducibility
    ///
    /// # Returns
    /// A tuple of (LWE instance, secret key)
    pub fn generate(
        n: usize,
        m: usize,
        q: i32,
        distribution: LweErrorDistribution,
        seed: u64,
    ) -> (Self, LweSecretKey) {
        let mut rng_state = seed;
        
        // Generate random matrix A
        let matrix_a: Vec<Vec<i32>> = (0..m)
            .map(|_| {
                (0..n)
                    .map(|_| {
                        rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
                        ((rng_state >> 33) as i32) % q
                    })
                    .collect()
            })
            .collect();

        // Generate secret vector s (small entries)
        let secret: Vec<i32> = (0..n)
            .map(|_| {
                rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
                sample_error(&mut rng_state, distribution, q)
            })
            .collect();

        // Generate error vector e
        let error: Vec<i32> = (0..m)
            .map(|_| {
                rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
                sample_error(&mut rng_state, distribution, q)
            })
            .collect();

        // Compute b = A·s + e (mod q)
        let vector_b: Vec<i32> = (0..m)
            .map(|i| {
                let as_i: i32 = matrix_a[i]
                    .iter()
                    .zip(secret.iter())
                    .map(|(&a, &s)| (a as i64 * s as i64) % (q as i64))
                    .sum::<i64>() as i32;
                (as_i + error[i]).rem_euclid(q)
            })
            .collect();

        let instance = LweInstance {
            matrix_a,
            vector_b,
            n,
            m,
            q,
        };

        let secret_key = LweSecretKey {
            secret,
            n,
            q,
        };

        (instance, secret_key)
    }

    /// Checks if a candidate secret is correct
    pub fn verify_secret(&self, candidate: &[i32], error_bound: i32) -> bool {
        if candidate.len() != self.n {
            return false;
        }

        // Compute A·candidate and check if b - A·candidate has small entries
        for i in 0..self.m {
            let as_i: i32 = self.matrix_a[i]
                .iter()
                .zip(candidate.iter())
                .map(|(&a, &s)| (a as i64 * s as i64) % (self.q as i64))
                .sum::<i64>() as i32;
            
            let diff = (self.vector_b[i] - as_i).rem_euclid(self.q);
            let centered = if diff > self.q / 2 { diff - self.q } else { diff };
            
            if centered.abs() > error_bound {
                return false;
            }
        }
        true
    }
}

/// Sample from error distribution
fn sample_error(rng_state: &mut u64, distribution: LweErrorDistribution, q: i32) -> i32 {
    match distribution {
        LweErrorDistribution::CenteredBinomial { eta } => {
            let mut sum = 0i32;
            for _ in 0..(2 * eta) {
                *rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
                if (*rng_state >> 63) == 1 {
                    sum += 1;
                }
            }
            (sum - eta as i32).rem_euclid(q)
        }
        LweErrorDistribution::DiscreteGaussian { sigma } => {
            // Box-Muller approximation (simplified)
            *rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let u1 = (*rng_state as f64) / (u64::MAX as f64);
            *rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let u2 = (*rng_state as f64) / (u64::MAX as f64);
            
            let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
            let sample = (z * sigma).round() as i32;
            sample.rem_euclid(q)
        }
        LweErrorDistribution::Uniform { bound } => {
            *rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let range = 2 * bound + 1;
            let sample = ((*rng_state >> 33) as i32 % range) - bound;
            sample.rem_euclid(q)
        }
    }
}

/// Ring-LWE Instance (polynomial version)
///
/// This is the efficient version used in practical schemes like ML-KEM.
/// Instead of matrices, we use polynomials in R_q = Z_q[X]/(X^n + 1).
#[derive(Debug, Clone)]
pub struct RingLweInstance {
    /// Public polynomial a ∈ R_q
    pub poly_a: Polynomial,
    /// Public polynomial b = a·s + e ∈ R_q
    pub poly_b: Polynomial,
}

/// Ring-LWE Secret Key
#[derive(Debug, Clone)]
pub struct RingLweSecretKey {
    /// Secret polynomial s ∈ R_q (small coefficients)
    pub secret: Polynomial,
}

impl RingLweInstance {
    /// Generates a Ring-LWE instance
    ///
    /// # Arguments
    /// * `seed` - Random seed
    /// * `eta` - Parameter for centered binomial distribution
    ///
    /// # Returns
    /// (Ring-LWE instance, secret key)
    pub fn generate(seed: u64, eta: usize) -> (Self, RingLweSecretKey) {
        // Generate random public polynomial a
        let poly_a = Polynomial::random(seed);
        
        // Generate small secret s
        let secret = Polynomial::sample_cbd(seed.wrapping_add(1000), eta);
        
        // Generate small error e
        let error = Polynomial::sample_cbd(seed.wrapping_add(2000), eta);
        
        // Compute b = a·s + e
        let as_product = poly_a.mul(&secret);
        let poly_b = as_product.add(&error);
        
        let instance = RingLweInstance { poly_a, poly_b };
        let secret_key = RingLweSecretKey { secret };
        
        (instance, secret_key)
    }

    /// Encrypts a single bit using Ring-LWE
    ///
    /// # Arguments
    /// * `message_bit` - The bit to encrypt (0 or 1)
    /// * `seed` - Random seed for encryption randomness
    /// * `eta` - CBD parameter
    ///
    /// # Returns
    /// Ciphertext (u, v) where u = a·r + e1 and v = b·r + e2 + m·⌊q/2⌋
    pub fn encrypt(&self, message_bit: u8, seed: u64, eta: usize) -> (Polynomial, Polynomial) {
        // Sample random r, e1, e2
        let r = Polynomial::sample_cbd(seed, eta);
        let e1 = Polynomial::sample_cbd(seed.wrapping_add(100), eta);
        let e2 = Polynomial::sample_cbd(seed.wrapping_add(200), eta);
        
        // u = a·r + e1
        let u = self.poly_a.mul(&r).add(&e1);
        
        // v = b·r + e2 + m·⌊q/2⌋
        let mut v = self.poly_b.mul(&r).add(&e2);
        
        // Add message encoding (only to constant coefficient)
        if message_bit == 1 {
            let q_half = MLKEM_Q / 2;
            v.coeffs[0] = (v.coeffs[0] + q_half).rem_euclid(MLKEM_Q);
        }
        
        (u, v)
    }
}

impl RingLweSecretKey {
    /// Decrypts a Ring-LWE ciphertext
    ///
    /// # Arguments
    /// * `u` - First ciphertext component
    /// * `v` - Second ciphertext component
    ///
    /// # Returns
    /// The decrypted bit (0 or 1)
    pub fn decrypt(&self, u: &Polynomial, v: &Polynomial) -> u8 {
        // Compute v - s·u
        let su = self.secret.mul(u);
        let diff = v.sub(&su);
        
        // Decode: check if coefficient is closer to 0 or q/2
        let coeff = diff.coeffs[0];
        let q_half = MLKEM_Q / 2;
        let _q_quarter = MLKEM_Q / 4;
        
        // If |coeff - q/2| < q/4, then message was 1
        let dist_to_half = ((coeff - q_half).abs()).min((coeff - q_half + MLKEM_Q).abs());
        let dist_to_zero = coeff.min(MLKEM_Q - coeff);
        
        if dist_to_half < dist_to_zero {
            1
        } else {
            0
        }
    }
}

/// Module-LWE Instance (used in ML-KEM and ML-DSA)
///
/// This generalizes Ring-LWE to vectors of polynomials for configurable security.
#[derive(Debug, Clone)]
pub struct ModuleLweInstance {
    /// Public matrix A ∈ R_q^(k×k)
    pub matrix_a: PolynomialMatrix,
    /// Public vector t = A·s + e ∈ R_q^k
    pub vector_t: PolynomialVector,
    /// Module rank k
    pub k: usize,
}

/// Module-LWE Secret Key
#[derive(Debug, Clone)]
pub struct ModuleLweSecretKey {
    /// Secret vector s ∈ R_q^k (small polynomials)
    pub secret: PolynomialVector,
}

impl ModuleLweInstance {
    /// Generates a Module-LWE instance
    ///
    /// # Arguments
    /// * `k` - Module rank (2 for ML-KEM-512, 3 for ML-KEM-768, 4 for ML-KEM-1024)
    /// * `seed` - Random seed
    /// * `eta` - CBD parameter
    pub fn generate(k: usize, seed: u64, eta: usize) -> (Self, ModuleLweSecretKey) {
        // Generate random public matrix A
        let matrix_a = PolynomialMatrix::random(k, seed);
        
        // Generate small secret vector s
        let secret = PolynomialVector::sample_cbd(k, seed.wrapping_add(100000), eta);
        
        // Generate small error vector e
        let error = PolynomialVector::sample_cbd(k, seed.wrapping_add(200000), eta);
        
        // Compute t = A·s + e
        let as_product = matrix_a.mul_vector(&secret);
        let vector_t = as_product.add(&error);
        
        let instance = ModuleLweInstance { matrix_a, vector_t, k };
        let secret_key = ModuleLweSecretKey { secret };
        
        (instance, secret_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lwe_generation() {
        let (instance, secret_key) = LweInstance::generate(
            16,  // n
            32,  // m
            3329, // q
            LweErrorDistribution::CenteredBinomial { eta: 2 },
            12345,
        );
        
        assert_eq!(instance.n, 16);
        assert_eq!(instance.m, 32);
        assert_eq!(secret_key.secret.len(), 16);
    }

    #[test]
    fn test_lwe_verification() {
        let (instance, secret_key) = LweInstance::generate(
            8, 16, 3329,
            LweErrorDistribution::CenteredBinomial { eta: 2 },
            54321,
        );
        
        // Correct secret should verify
        assert!(instance.verify_secret(&secret_key.secret, 10));
        
        // Wrong secret should not verify
        let wrong_secret: Vec<i32> = vec![0; 8];
        assert!(!instance.verify_secret(&wrong_secret, 2));
    }

    #[test]
    fn test_ring_lwe_encrypt_decrypt() {
        let (instance, secret_key) = RingLweInstance::generate(12345, 2);
        
        // Encrypt and decrypt bit 0
        let (u0, v0) = instance.encrypt(0, 11111, 2);
        let decrypted0 = secret_key.decrypt(&u0, &v0);
        assert_eq!(decrypted0, 0, "Failed to decrypt 0");
        
        // Encrypt and decrypt bit 1
        let (u1, v1) = instance.encrypt(1, 22222, 2);
        let decrypted1 = secret_key.decrypt(&u1, &v1);
        assert_eq!(decrypted1, 1, "Failed to decrypt 1");
    }

    #[test]
    fn test_module_lwe_generation() {
        for k in [2, 3, 4] {
            let (instance, secret_key) = ModuleLweInstance::generate(k, 99999, 2);
            
            assert_eq!(instance.k, k);
            assert_eq!(instance.vector_t.len(), k);
            assert_eq!(secret_key.secret.len(), k);
        }
    }
}
