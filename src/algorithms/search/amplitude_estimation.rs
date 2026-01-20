//! # Quantum Amplitude Estimation - Precision Probability Estimation
//!
//! ## 🎯 Purpose & Motivation
//!
//! Quantum Amplitude Estimation (QAE) provides **quadratic speedup** for estimating
//! the probability p that a quantum algorithm succeeds. Classically, achieving
//! precision ε requires O(1/ε²) samples; quantum estimation needs only O(1/ε).
//!
//! ## ⚙️ How It Works
//!
//! ### The Problem
//!
//! Given a unitary A that prepares state A|0⟩ = √(1-p)|ψ₀⟩ + √p|ψ₁⟩,
//! estimate p = sin²(θ) where θ is defined by the state preparation.
//!
//! ### Algorithm (Based on Quantum Phase Estimation)
//!
//! 1. Prepare superposition of Grover iterations G^k
//! 2. Use QPE to estimate eigenphase θ
//! 3. Extract p = sin²(πθ)
//!
//! ## 📊 Complexity
//!
//! - **Query Complexity**: O(1/ε) for precision ε
//! - **Classical Comparison**: O(1/ε²)
//! - **Speedup**: Quadratic
//!
//! ## 🔬 2025-26 Research Context
//!
//! - **Iterative QAE**: Uses O(log(1/ε)) qubits instead of O(1/ε)
//! - **Maximum Likelihood QAE**: Classical post-processing enhancement
//! - **Variational QAE**: Hybrid approaches for NISQ devices
//!
//! ## 📚 References
//!
//! - Brassard et al. (2002). "Quantum Amplitude Amplification and Estimation"
//! - Suzuki et al. (2020). "Amplitude estimation without phase estimation"

use std::f64::consts::PI;

/// Configuration for Amplitude Estimation
#[derive(Debug, Clone)]
pub struct AmplitudeEstimationConfig {
    /// Number of precision qubits (m)
    pub precision_qubits: usize,
    /// Target precision ε (alternative to specifying qubits)
    pub target_precision: Option<f64>,
}

impl AmplitudeEstimationConfig {
    /// Creates config with specified precision qubits
    pub fn with_qubits(m: usize) -> Self {
        AmplitudeEstimationConfig {
            precision_qubits: m,
            target_precision: Some(PI / (1 << m) as f64),
        }
    }

    /// Creates config to achieve target precision
    pub fn with_precision(epsilon: f64) -> Self {
        // m ≈ log₂(π/ε) + 1
        let m = ((PI / epsilon).log2().ceil() as usize).max(1);
        AmplitudeEstimationConfig {
            precision_qubits: m,
            target_precision: Some(epsilon),
        }
    }

    /// Returns expected number of oracle queries
    pub fn query_count(&self) -> usize {
        // QAE uses O(2^m) queries
        1 << self.precision_qubits
    }
}

/// Result of Amplitude Estimation
#[derive(Debug, Clone)]
pub struct AmplitudeEstimationResult {
    /// Estimated amplitude a = sin(θ)
    pub amplitude: f64,
    /// Estimated probability p = a² = sin²(θ)
    pub probability: f64,
    /// Estimated phase θ
    pub phase: f64,
    /// Confidence interval (lower, upper)
    pub confidence_interval: (f64, f64),
    /// Number of oracle queries used
    pub queries_used: usize,
}

/// State for Amplitude Estimation simulation
#[derive(Debug, Clone)]
pub struct AmplitudeEstimator {
    /// Number of system qubits
    pub n: usize,
    /// Success probability p (for oracle simulation)
    pub true_probability: f64,
}

impl AmplitudeEstimator {
    /// Creates an estimator for a given success probability
    ///
    /// In practice, this would be connected to an actual quantum oracle.
    /// Here we simulate knowing the true probability for testing.
    pub fn new(n: usize, true_probability: f64) -> Self {
        AmplitudeEstimator {
            n,
            true_probability: true_probability.clamp(0.0, 1.0),
        }
    }

    /// Simulates Quantum Amplitude Estimation
    ///
    /// This is a simplified simulation that demonstrates the algorithm's
    /// output without full QPE circuit simulation.
    pub fn estimate(&self, config: &AmplitudeEstimationConfig) -> AmplitudeEstimationResult {
        let m = config.precision_qubits;
        let num_states = 1 << m;
        
        // True phase: θ where p = sin²(πθ/2)
        let true_theta = self.true_probability.sqrt().asin();
        
        // In QAE, we estimate θ/π with m bits of precision
        // The measured value is k/2^m where k is closest to θ*2^m/π
        let theta_scaled = true_theta * (num_states as f64) / PI;
        let k = theta_scaled.round() as i64;
        let k = k.rem_euclid(num_states as i64) as usize;
        
        // Estimated phase
        let estimated_phase = (k as f64) * PI / (num_states as f64);
        
        // Estimated amplitude and probability
        let estimated_amplitude = estimated_phase.sin();
        let estimated_probability = estimated_amplitude.powi(2);
        
        // Confidence interval based on precision
        let delta = PI / (num_states as f64);
        let lower_prob = (estimated_phase - delta).max(0.0).sin().powi(2);
        let upper_prob = (estimated_phase + delta).min(PI / 2.0).sin().powi(2);
        
        AmplitudeEstimationResult {
            amplitude: estimated_amplitude,
            probability: estimated_probability,
            phase: estimated_phase,
            confidence_interval: (lower_prob, upper_prob),
            queries_used: num_states,
        }
    }
}

/// Iterative Quantum Amplitude Estimation (IQAE)
///
/// A more NISQ-friendly variant that uses fewer qubits but more classical
/// processing, based on Suzuki et al. (2020).
#[derive(Debug, Clone)]
pub struct IterativeAmplitudeEstimator {
    /// Number of system qubits
    pub n: usize,
    /// True probability (for simulation)
    pub true_probability: f64,
}

impl IterativeAmplitudeEstimator {
    pub fn new(n: usize, true_probability: f64) -> Self {
        IterativeAmplitudeEstimator {
            n,
            true_probability: true_probability.clamp(0.0, 1.0),
        }
    }

    /// Runs Iterative Amplitude Estimation
    ///
    /// Uses a single ancilla qubit and iteratively refines the estimate
    /// using Grover iterations with different powers.
    ///
    /// # Arguments
    /// * `target_precision` - Desired precision ε
    /// * `confidence` - Confidence level (e.g., 0.95 for 95%)
    ///
    /// # Returns
    /// Amplitude estimation result with the final estimate
    pub fn estimate_iterative(
        &self,
        target_precision: f64,
        _confidence: f64,
    ) -> AmplitudeEstimationResult {
        // Number of iterations needed: O(log(1/ε))
        let num_iterations = ((1.0 / target_precision).log2().ceil() as usize).max(1);
        
        // Start with initial bounds [0, 1]
        let mut theta_lower = 0.0;
        let mut theta_upper = PI / 2.0;
        
        let mut total_queries = 0;
        
        for k in 0..num_iterations {
            let num_grover_ops = 1 << k;  // 2^k Grover iterations
            total_queries += num_grover_ops;
            
            // Simulate measurement outcome
            // In real implementation, would run circuit and measure
            let true_theta = self.true_probability.sqrt().asin();
            let measured_angle = 2.0 * (2 * num_grover_ops + 1) as f64 * true_theta;
            
            // Update bounds based on measurement
            let midpoint = (theta_lower + theta_upper) / 2.0;
            if measured_angle.sin().powi(2) > midpoint.sin().powi(2) {
                theta_lower = midpoint;
            } else {
                theta_upper = midpoint;
            }
        }
        
        let estimated_phase = (theta_lower + theta_upper) / 2.0;
        let estimated_amplitude = estimated_phase.sin();
        let estimated_probability = estimated_amplitude.powi(2);
        
        AmplitudeEstimationResult {
            amplitude: estimated_amplitude,
            probability: estimated_probability,
            phase: estimated_phase,
            confidence_interval: (theta_lower.sin().powi(2), theta_upper.sin().powi(2)),
            queries_used: total_queries,
        }
    }
}

/// Maximum Likelihood Amplitude Estimation (MLAE)
///
/// Uses classical post-processing to extract better estimates
/// from limited quantum measurements, per Suzuki et al. (2020).
#[derive(Debug, Clone)]
pub struct MaxLikelihoodEstimator {
    /// Measurement outcomes: (k, num_ones, num_total) for Grover power k
    pub measurements: Vec<(usize, usize, usize)>,
}

impl MaxLikelihoodEstimator {
    pub fn new() -> Self {
        MaxLikelihoodEstimator {
            measurements: Vec::new(),
        }
    }

    /// Adds a measurement result
    ///
    /// # Arguments
    /// * `grover_iterations` - Number of Grover iterations used
    /// * `observed_ones` - Number of times "1" (success) was measured
    /// * `total_shots` - Total number of measurements
    pub fn add_measurement(&mut self, grover_iterations: usize, observed_ones: usize, total_shots: usize) {
        self.measurements.push((grover_iterations, observed_ones, total_shots));
    }

    /// Computes the maximum likelihood estimate
    ///
    /// Finds θ that maximizes the likelihood of observed measurements.
    pub fn compute_estimate(&self) -> AmplitudeEstimationResult {
        if self.measurements.is_empty() {
            return AmplitudeEstimationResult {
                amplitude: 0.0,
                probability: 0.0,
                phase: 0.0,
                confidence_interval: (0.0, 1.0),
                queries_used: 0,
            };
        }

        // Grid search for maximum likelihood
        let num_points = 1000;
        let mut best_theta = 0.0;
        let mut best_log_likelihood = f64::NEG_INFINITY;
        
        for i in 0..=num_points {
            let theta = (i as f64 / num_points as f64) * PI / 2.0;
            
            let mut log_likelihood = 0.0;
            for &(k, ones, total) in &self.measurements {
                // Probability of success after k Grover iterations
                let p_k = ((2 * k + 1) as f64 * theta).sin().powi(2);
                let p_k = p_k.clamp(1e-10, 1.0 - 1e-10);
                
                // Binomial log-likelihood
                log_likelihood += (ones as f64) * p_k.ln() 
                    + ((total - ones) as f64) * (1.0 - p_k).ln();
            }
            
            if log_likelihood > best_log_likelihood {
                best_log_likelihood = log_likelihood;
                best_theta = theta;
            }
        }
        
        let total_queries: usize = self.measurements.iter()
            .map(|&(k, _, shots)| (2 * k + 1) * shots)
            .sum();
        
        AmplitudeEstimationResult {
            amplitude: best_theta.sin(),
            probability: best_theta.sin().powi(2),
            phase: best_theta,
            confidence_interval: (
                (best_theta - 0.05).max(0.0).sin().powi(2),
                (best_theta + 0.05).min(PI / 2.0).sin().powi(2),
            ),
            queries_used: total_queries,
        }
    }
}

impl Default for MaxLikelihoodEstimator {
    fn default() -> Self {
        Self::new()
    }
}

/// Runs standard Quantum Amplitude Estimation
///
/// # Arguments
/// * `n` - Number of system qubits
/// * `true_prob` - True success probability (for simulation)
/// * `precision_qubits` - Number of ancilla qubits for precision
///
/// # Example
///
/// ```
/// use quantic_rust::algorithms::search::amplitude_estimation;
///
/// let result = amplitude_estimation(4, 0.3, 6);
/// assert!((result.probability - 0.3).abs() < 0.1);
/// ```
pub fn amplitude_estimation(n: usize, true_prob: f64, precision_qubits: usize) -> AmplitudeEstimationResult {
    let estimator = AmplitudeEstimator::new(n, true_prob);
    let config = AmplitudeEstimationConfig::with_qubits(precision_qubits);
    estimator.estimate(&config)
}

/// Runs Iterative Amplitude Estimation (NISQ-friendly)
pub fn iterative_amplitude_estimation(
    n: usize,
    true_prob: f64,
    precision: f64,
) -> AmplitudeEstimationResult {
    let estimator = IterativeAmplitudeEstimator::new(n, true_prob);
    estimator.estimate_iterative(precision, 0.95)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_amplitude_estimation_basic() {
        let result = amplitude_estimation(4, 0.25, 4);
        
        // Should be within reasonable precision
        assert!((result.probability - 0.25).abs() < 0.15);
    }

    #[test]
    fn test_amplitude_estimation_high_precision() {
        let result = amplitude_estimation(4, 0.5, 8);
        
        assert!((result.probability - 0.5).abs() < 0.05);
    }

    #[test]
    fn test_iterative_estimation() {
        let result = iterative_amplitude_estimation(4, 0.3, 0.05);
        
        assert!((result.probability - 0.3).abs() < 0.15);
        assert!(result.queries_used > 0);
    }

    #[test]
    fn test_config_with_precision() {
        let config = AmplitudeEstimationConfig::with_precision(0.01);
        
        assert!(config.precision_qubits >= 8);  // Need ~9 qubits for 0.01 precision
    }

    #[test]
    fn test_max_likelihood_estimator() {
        let mut mle = MaxLikelihoodEstimator::new();
        
        // Simulate measurements for θ ≈ π/6 (p ≈ 0.25)
        mle.add_measurement(0, 25, 100);   // k=0: measure original
        mle.add_measurement(1, 75, 100);   // k=1: after Grover
        
        let result = mle.compute_estimate();
        
        assert!(result.probability > 0.0);
        assert!(result.probability < 1.0);
    }

    #[test]
    fn test_confidence_interval() {
        let result = amplitude_estimation(4, 0.4, 5);
        
        let (lower, upper) = result.confidence_interval;
        assert!(lower <= result.probability);
        assert!(upper >= result.probability);
    }

    #[test]
    fn test_edge_cases() {
        // Very small probability
        let result = amplitude_estimation(4, 0.01, 6);
        assert!(result.probability < 0.1);
        
        // Very high probability
        let result = amplitude_estimation(4, 0.99, 6);
        assert!(result.probability > 0.9);
    }
}
