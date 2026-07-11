//! Simulation configuration.
//!
//! Holds the handful of *fundamental* parameters. Every per-cell division
//! threshold (timer period, sizer target, adder increment) is **derived** from
//! these via the helper methods below — so the critical time/size/increment are
//! never configured directly. Configure `r` and `V₀`; the thresholds follow.

#[derive(Debug, Clone)]
pub struct Config {
    /// Exponential growth rate `r` in `V(t) = V_b · e^(r·t)`.
    pub growth_rate: f64,
    /// Characteristic / initial birth volume `V₀` — sets the size scale.
    pub initial_volume: f64,
    /// Simulation timestep.
    pub dt: f64,
    /// Noise 1 — partitioning asymmetry. Std dev of the split fraction `f`,
    /// drawn from `Normal(0.5, split_noise)`. This is the *driver* of size
    /// variability: at 0.0 division is perfectly symmetric and nothing varies.
    pub split_noise: f64,
    /// Noise 2 — cell-to-cell variability in the division threshold, as a
    /// coefficient of variation (`σ = CV · mean`). Adds steady-state scatter.
    pub threshold_noise_cv: f64,
    /// RNG seed. Fixed by default so every run is reproducible; change it (or
    /// expose it via CLI later) to explore different stochastic realisations.
    pub seed: u64,
    pub alpha: f64,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            growth_rate: 0.5,
            initial_volume: 1.0,
            dt: 0.02,
            split_noise: 0.05,
            threshold_noise_cv: 0.1,
            seed: 42,
            alpha: 0.0,
        }
    }
}

impl Config {
    /// Timer period that yields one doubling per cycle: `τ = ln(2) / r`.
    pub fn timer_period(&self) -> f64 {
        std::f64::consts::LN_2 / self.growth_rate
    }

    /// Sizer target size that yields a doubling on average: `2·V₀`.
    pub fn sizer_target(&self) -> f64 {
        2.0 * self.initial_volume
    }

    /// Adder increment that yields a doubling on average: `V₀`.
    pub fn adder_increment(&self) -> f64 {
        self.initial_volume
    }
    /// equilibriul euqation to set division volume
    pub fn v_c(&self) -> f64 {
        (1.0 - self.alpha)*self.initial_volume
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timer_period_doubles_volume() {
        // Over one timer period, volume grows by e^(r·τ). With τ = ln(2)/r
        // that factor must be exactly 2.
        let cfg = Config::default();
        let tau = cfg.timer_period();
        let factor = (cfg.growth_rate * tau).exp();
        assert!((factor - 2.0).abs() < 1e-9, "factor = {}", factor);
    }
}
