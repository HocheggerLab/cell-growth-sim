use crate::cell::{Cell, SizeControlModel};
use crate::config::Config;
use rand::Rng;
use serde::Serialize;



pub fn growth_loop(
    cell_array: Vec<Cell>,
    cfg: &Config,
    rng: &mut impl Rng,
    tick: u64,
) -> (Vec<Cell>, Vec<DivisionEvent>) {
    let mut new_cell_array = vec![];
    let mut events = vec![];
    let dt = cfg.dt;
    let r = cfg.growth_rate;
    let time = tick as f64 * dt;
    for mut cell in cell_array {
        cell.grow(dt, r);
        if cell.ready_to_divide() {
            let (cell1, cell2) = cell.divide(cfg, rng);
            events.push(DivisionEvent::new(&cell, &cell1, &cell2, time)); // borrow before the move
            new_cell_array.extend([cell1, cell2]);
        } else {
            new_cell_array.push(cell);
        }
    }
    (new_cell_array, events)
}

#[derive(Debug, Clone, Serialize)]
pub struct DivisionEvent {
    pub birth_volume: f64,
    pub division_volume: f64,
    pub added_volume: f64,
    pub generation_time: f64,
    pub generation: u64,
    pub daughter_volumes: (f64, f64),
    pub time: f64
}

impl DivisionEvent {
    pub fn new(cell: &Cell, cell1: &Cell, cell2: &Cell, time: f64) -> Self {
        DivisionEvent {
            birth_volume: cell.birth_volume(),
            division_volume: cell.volume(),
            added_volume: cell.volume() - cell.birth_volume(),
            generation_time: cell.age(),
            generation: cell.generation(),
            daughter_volumes: (cell1.birth_volume(), cell2.birth_volume()),
            time
        }
    }
}

pub fn run(
    model: SizeControlModel, 
    cfg: &Config, 
    rng: &mut impl Rng,
    n_max: usize,
) -> Vec<DivisionEvent> {
    let mut cell_array = vec![Cell::new(cfg.initial_volume, 0, model)];
    let mut all_events = vec![];
    let mut tick: u64 = 0;
    while cell_array.len() < n_max {
        let (next_pop, tick_events) = growth_loop(cell_array, cfg, rng, tick);
        cell_array = next_pop;            // reassign the OUTER variable (no `let`)
        all_events.extend(tick_events);
        tick += 1;
        }
    all_events 
    }

pub fn events_to_json(events: &[DivisionEvent]) -> serde_json::Result<String> {
    serde_json::to_string_pretty(events)
}


#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn test_growth_loop_volume() {
        let cfg = Config::default();
        let mut rng = StdRng::seed_from_u64(cfg.seed);
        let cell1 = Cell::new(20.0, 0, SizeControlModel::Timer { period: 20.0 });
        let cell2 = Cell::new(21.0, 0, SizeControlModel::Timer { period: 20.0 });
        let cell_array = vec![cell1, cell2];
        let (new_cell_array, _events) = growth_loop(cell_array, &cfg, &mut rng, 0);
        assert!(new_cell_array[0].volume() > 21.0254219);
        assert!(new_cell_array[1].volume() > 22.0766930);
    }
    
    #[test]
    fn timer_cell_divides_in_loop() {
      let cfg = Config::default();
      let mut rng = StdRng::seed_from_u64(cfg.seed);

      let cell = Cell::new(20.0, 0, SizeControlModel::Timer { period: 0.05 });
      let (pop, _events) = growth_loop(vec![cell], &cfg, &mut rng, 0);

      assert_eq!(pop.len(), 2, "timer past period should split into two");
  }
  #[test]
    fn sizer_cell_divides_in_loop() {
      let cfg = Config::default();
      let mut rng = StdRng::seed_from_u64(cfg.seed);

      let cell = Cell::new(20.0, 0, SizeControlModel::Sizer { target_size: 20.05 });
      let (pop, _events) = growth_loop(vec![cell], &cfg, &mut rng, 0);

      assert_eq!(pop.len(), 2, "timer past period should split into two");
  }
  #[test]
    fn adder_cell_divides_in_loop() {
      let cfg = Config::default();
      let mut rng = StdRng::seed_from_u64(cfg.seed);

      let cell = Cell::new(20.0, 0, SizeControlModel::Adder { increment: 0.05 });
      let (pop, _events) = growth_loop(vec![cell], &cfg, &mut rng, 0);

      assert_eq!(pop.len(), 2, "timer past period should split into two");
  }
}
