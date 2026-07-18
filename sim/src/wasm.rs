use wasm_bindgen::prelude::*;
use serde::Deserialize;
use rand::rngs::StdRng;
use rand::SeedableRng;

use crate::cell::SizeControlModel;
use crate::config::Config;
use crate::simulation::run;

// The parameters the browser will send in, as one JS object.
// serde matches these field names against the object's keys.
#[derive(Deserialize)]
struct SimParams {
    model: String,
    n_max: usize,
    split_noise: f64,
    threshold_noise_cv: f64,
    seed: u64,
    alpha: f64,
}

#[wasm_bindgen]
pub fn run_sim(params: JsValue) -> Result<JsValue, JsValue> {
    // 1. Decode the JS object into our struct.
    let p: SimParams = serde_wasm_bindgen::from_value(params)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;


    // 2. Build the Config: take the tunable fields, default the rest.
    let cfg = Config {
        split_noise: p.split_noise,
        threshold_noise_cv: p.threshold_noise_cv,
        seed: p.seed,
        alpha: p.alpha,
        ..Config::default()
    };

    // 3. Pick the model — same match as main.rs, but on a string.
    let model = match p.model.as_str() {
        "timer"       => SizeControlModel::Timer { period: cfg.timer_period() },
        "sizer"       => SizeControlModel::Sizer { target_size: cfg.sizer_target() },
        "adder"       => SizeControlModel::Adder { increment: cfg.adder_increment() },
        "adder-alpha" => SizeControlModel::AdderAlpha { alpha: cfg.alpha, v_c: cfg.v_c() },
        other => return Err(JsValue::from_str(&format!("unknown model: {other}"))),
    };

    // 4. Run and hand the events back to JS as a real array of objects.
    let mut rng = StdRng::seed_from_u64(cfg.seed);
    let events = run(model, &cfg, &mut rng, p.n_max);
    serde_wasm_bindgen::to_value(&events).map_err(|e| JsValue::from_str(&e.to_string()))
}

