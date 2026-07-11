use crate::config::Config;
use rand::Rng;
use rand_distr::{Distribution, Normal};

#[derive(Debug, Clone, Copy)]
pub enum SizeControlModel {
    Timer { period: f64 },
    Sizer { target_size: f64 },
    Adder { increment: f64 },
    AdderAlpha { alpha: f64, v_c: f64 },
}

impl SizeControlModel{
    pub fn draw(kind: ModelKind, cfg: &Config, rng: &mut impl Rng) -> Self {
        let cv = cfg.threshold_noise_cv;
        match kind {
            ModelKind::Timer => SizeControlModel::Timer {
                period: Normal::new(cfg.timer_period(), cv * cfg.timer_period()).unwrap().sample(rng),
            },
            ModelKind::Sizer => SizeControlModel::Sizer {
                target_size: Normal::new(cfg.sizer_target(), cv * cfg.sizer_target()).unwrap().sample(rng)
            },
            ModelKind::Adder => SizeControlModel::Adder {
                increment: Normal::new(cfg.adder_increment(), cv * cfg.adder_increment()).unwrap().sample(rng)
            },
            ModelKind::AdderAlpha => SizeControlModel::AdderAlpha {
                alpha: cfg.alpha, v_c: Normal::new(cfg.v_c(), cv * cfg.v_c()).unwrap().sample(rng),
            } 
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelKind {
    Timer,
    Sizer,
    Adder,
    AdderAlpha,
}


#[derive(Debug, Clone)]
pub struct Cell {
    birth_volume: f64,
    volume: f64,
    age: f64,
    generation:u64,
    size_control_model: SizeControlModel,
}

impl Cell {
    pub fn new(birth_volume: f64, generation: u64, size_control_model: SizeControlModel) -> Self {
        Cell {
            birth_volume,
            volume: birth_volume,
            age: 0.0,
            generation,
            size_control_model,
        }
    }

    pub fn grow(&mut self, dt: f64, r: f64) {
        self.volume *= (dt * r).exp();
        self.age += dt;
    }

    pub fn volume(&self) -> f64 {
        self.volume
    }

    pub fn age(&self) -> f64 {
        self.age
    }

    pub fn birth_volume(&self) -> f64 {
        self.birth_volume
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn ready_to_divide(&self) -> bool {
        match self.size_control_model {
            SizeControlModel::Timer { period } => self.age >= period,
            SizeControlModel::Sizer { target_size } => self.volume >= target_size,
            SizeControlModel::Adder { increment } => self.volume - self.birth_volume >= increment,
            SizeControlModel::AdderAlpha { alpha, v_c } => self.volume - self.birth_volume >= alpha * self.birth_volume + v_c,
        }
    }

    pub fn kind(&self) -> ModelKind {
        match self.size_control_model {
            SizeControlModel::Timer { .. } => ModelKind::Timer,
            SizeControlModel::Sizer { .. } => ModelKind::Sizer,
            SizeControlModel::Adder { .. } => ModelKind::Adder,
            SizeControlModel::AdderAlpha { .. } => ModelKind::AdderAlpha,
        }
    }

    pub fn divide(&self, cfg: &Config, rng: &mut impl Rng) -> (Cell, Cell) {
        let f = Normal::new(0.5, cfg.split_noise).unwrap().sample(rng).clamp(0.05, 0.95);
        let v1 = self.volume * f;
        let v2 = self.volume * (1.0 - f);
        let generation = self.generation + 1; 
        let cell1 = Cell::new(v1, generation, SizeControlModel::draw(self.kind(), cfg, rng));
        let cell2 = Cell::new(v2, generation, SizeControlModel::draw(self.kind(), cfg, rng));
        (cell1, cell2)
    }
}
    

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn test_new_cell() {
        let c = Cell::new(20.0, 0, SizeControlModel::Timer { period: 20.0 });
        assert_eq!(c.volume, 20.0);
    }

    #[test]
    fn test_grow_cell() {
        let mut c = Cell::new(1.0, 0, SizeControlModel::Adder { increment: 2.0 });
        c.grow(1.0, 0.1);
        let expected = 1.1051709;
        let actual = c.volume;
        assert!(
            (actual - expected).abs() < 1e-6,
            "expected ≈{}, got {}",
            expected, actual,
        );
    }

    // --- ready_to_divide -----------------------------------------------------
    //
    // `ready_to_divide` is a PURE predicate: no RNG, no Config. So we just build
    // a cell and put it into an exact state, then check the yes/no answer.
    //
    // We set private fields (`age`, `volume`) directly. That's allowed because
    // `tests` is a CHILD module of the module that defines `Cell`, and child
    // modules can see their parent's private items (see the per-module privacy
    // point in the getters/privacy note). It also lets us hit the threshold
    // exactly — which matters because the criterion is `>=`.

    #[test]
    fn timer_divides_when_age_reaches_period() {
        let mut c = Cell::new(1.0, 0, SizeControlModel::Timer { period: 5.0 });

        c.age = 4.9;
        assert!(!c.ready_to_divide(), "below period: should NOT divide");

        c.age = 5.0;
        assert!(c.ready_to_divide(), "exactly at period: should divide (>=)");

        c.age = 5.1;
        assert!(c.ready_to_divide(), "past period: should divide");
    }

    #[test]
    fn sizer_divides_when_volume_reaches_target() {
        // The sizer cares only about absolute volume — age and birth_volume
        // are irrelevant to it.
        let mut c = Cell::new(1.0, 0, SizeControlModel::Sizer { target_size: 2.0 });

        c.volume = 1.99;
        assert!(!c.ready_to_divide());

        c.volume = 2.0;
        assert!(c.ready_to_divide());
    }

    #[test]
    fn adder_fires_on_added_volume_independent_of_birth_size() {
        // The DEFINING property of the adder: it divides after adding a fixed
        // increment, regardless of how big the cell was at birth. We test that
        // by giving two very different birth sizes the SAME increment and
        // checking they both fire after adding exactly `increment`.
        let increment = 1.0;
        let mut small = Cell::new(1.0, 0, SizeControlModel::Adder { increment });
        let mut big = Cell::new(10.0, 0, SizeControlModel::Adder { increment });

        small.volume = 1.0 + 0.99; // added 0.99 < 1.0
        big.volume = 10.0 + 0.99;
        assert!(!small.ready_to_divide());
        assert!(!big.ready_to_divide());

        small.volume = 1.0 + 1.0; // added exactly 1.0
        big.volume = 10.0 + 1.0;
        assert!(small.ready_to_divide());
        assert!(big.ready_to_divide(), "added volume is independent of birth size");
    }

    #[test]
    fn adder_alpha_reduces_to_pure_adder_when_alpha_is_zero() {
        // alpha = 0 → threshold is just v_c on the added volume → same as Adder.
        let mut c = Cell::new(3.0, 0, SizeControlModel::AdderAlpha { alpha: 0.0, v_c: 1.0 });

        c.volume = 3.0 + 0.5;
        assert!(!c.ready_to_divide());

        c.volume = 3.0 + 1.0;
        assert!(c.ready_to_divide());
    }

    #[test]
    fn adder_alpha_reduces_to_sizer_when_alpha_is_minus_one() {
        // alpha = -1 → (volume - birth) >= -birth + v_c → volume >= v_c.
        // The birth_volume terms cancel, leaving an absolute-size (sizer) rule.
        let mut c = Cell::new(3.0, 0, SizeControlModel::AdderAlpha { alpha: -1.0, v_c: 2.0 });

        c.volume = 1.99;
        assert!(!c.ready_to_divide());

        c.volume = 2.0;
        assert!(c.ready_to_divide(), "alpha = -1 collapses AdderAlpha to a sizer at v_c");
    }

    // --- divide (stochastic) -------------------------------------------------
    //
    // divide() uses the RNG, so we seed a StdRng for reproducibility. The
    // strongest test is the seed-INDEPENDENT invariant: whatever the split
    // fraction comes out as, the two daughters' volumes must sum to the
    // mother's. Growing the mother first pushes volume != birth_volume, so the
    // test also catches a bug where divide partitions birth_volume by mistake.

    #[test]
    fn division_conserves_volume() {
        let cfg = Config::default();
        let mut rng = StdRng::seed_from_u64(cfg.seed);

        let mut mother = Cell::new(4.0, 0, SizeControlModel::Adder { increment: 1.0 });
        mother.grow(cfg.dt, cfg.growth_rate); // volume now differs from birth_volume

        let (d1, d2) = mother.divide(&cfg, &mut rng);
        assert!(
            (d1.volume + d2.volume - mother.volume).abs() < 1e-12,
            "daughters {} + {} should sum to mother {}",
            d1.volume, d2.volume, mother.volume,
        );
    }

    // Invariant: daughters are born fresh — age 0 and volume == birth_volume.
    // (Cell::new guarantees this; the test confirms divide actually routes
    // through new rather than copying the mother's age/volume by mistake.)
    #[test]
    fn daughters_are_newborns() {
        let cfg = Config::default();
        let mut rng = StdRng::seed_from_u64(cfg.seed);

        let mut mother = Cell::new(4.0, 0, SizeControlModel::Timer { period: 1.0 });
        mother.grow(cfg.dt, cfg.growth_rate); // mother has age > 0, volume > birth

        let (d1, d2) = mother.divide(&cfg, &mut rng);
        for d in [&d1, &d2] {
            assert_eq!(d.age, 0.0, "daughter should start at age 0");
            assert_eq!(d.volume, d.birth_volume, "newborn volume == birth_volume");
        }
    }

    // Invariant: a daughter is the SAME kind as its mother (a sizer begets
    // sizers). Guards against divide accidentally changing the control model.
    #[test]
    fn daughters_inherit_mother_kind() {
        let cfg = Config::default();
        let mut rng = StdRng::seed_from_u64(cfg.seed);

        let mother = Cell::new(4.0, 0, SizeControlModel::Sizer { target_size: 8.0 });
        let (d1, d2) = mother.divide(&cfg, &mut rng);

        assert_eq!(d1.kind(), ModelKind::Sizer);
        assert_eq!(d2.kind(), ModelKind::Sizer);
    }

    // Reproducibility: the SAME seed must produce identical divisions. Note we
    // use EXACT equality (not a tolerance) — the whole point is bit-for-bit
    // identical results from the same random stream.
    #[test]
    fn division_is_reproducible_with_same_seed() {
        let cfg = Config::default();
        let mother = Cell::new(4.0, 0, SizeControlModel::Sizer { target_size: 8.0 });

        let mut rng_a = StdRng::seed_from_u64(123);
        let mut rng_b = StdRng::seed_from_u64(123);
        let (a1, a2) = mother.divide(&cfg, &mut rng_a);
        let (b1, b2) = mother.divide(&cfg, &mut rng_b);

        assert_eq!(a1.volume, b1.volume);
        assert_eq!(a2.volume, b2.volume);
    }

    // Statistical: over many divisions the mean split fraction should be ~0.5
    // (division is symmetric on average). We assert on the distribution's mean,
    // never on any single draw — and use a tolerance, since it's a sampled stat.
    #[test]
    fn mean_split_is_balanced_over_many_divisions() {
        let cfg = Config::default();
        let mut rng = StdRng::seed_from_u64(7);
        let mother = Cell::new(10.0, 0, SizeControlModel::Adder { increment: 1.0 });

        let n = 10_000;
        let mut sum_fraction = 0.0;
        for _ in 0..n {
            // divide takes &self, so we can re-divide the same mother repeatedly.
            let (d1, _d2) = mother.divide(&cfg, &mut rng);
            sum_fraction += d1.volume / mother.volume;
        }
        let mean = sum_fraction / n as f64;
        assert!((mean - 0.5).abs() < 0.01, "mean split fraction {} should be ~0.5", mean);
    }
}