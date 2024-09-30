use std::time::Duration;
use web_time::Instant;

use winit::event_loop::ControlFlow;

use super::{Game, timing_stats::TimingStats};



pub struct GameLoop {
    seconds_per_update: chrono::Duration,
    seconds_per_render: chrono::Duration,

    update_accumulator: chrono::Duration,
    render_accumulator: chrono::Duration,

    last_cycle: Instant,
    last_update: Instant,
    last_render: Instant,

    render_next: bool,
    update_next: bool,

    stats: TimingStats,
}

impl GameLoop {
    pub fn new(ups: f32, fps: f32) -> Self {
        let seconds_per_render = chrono::Duration::from_std(Duration::from_secs_f32(1.0 / fps)).unwrap();
        let seconds_per_update = chrono::Duration::from_std(Duration::from_secs_f32(1.0 / ups)).unwrap();

        Self {
            seconds_per_render,
            seconds_per_update,
            
            render_accumulator: chrono::Duration::zero(),
            update_accumulator: chrono::Duration::zero(),

            last_cycle: Instant::now(),
            last_update: Instant::now(),
            last_render: Instant::now(),

            render_next: false,
            update_next: false,

            stats: TimingStats::new(),
        }
    }

    pub fn default() -> Self {
        Self::new(15f32, 60f32)
    }

    // reset accumulators to zero; call right before turning on
    pub fn reset(&mut self) {
        self.render_accumulator = chrono::Duration::zero();
        self.update_accumulator = chrono::Duration::zero();
        
        self.last_cycle = Instant::now();
        self.last_update = Instant::now();
        self.last_render = Instant::now();

        self.render_next = false;
        self.update_next = false;

        self.stats.reset();
    }
    
    pub fn update_or_render<T: Game>(&mut self, game: &mut T) -> ControlFlow {
        // update accumulators
        let now = Instant::now();
        self.update_accumulator = self.update_accumulator + chrono::Duration::from_std(now - self.last_cycle).unwrap();
        self.render_accumulator = self.render_accumulator + chrono::Duration::from_std(now - self.last_cycle).unwrap();
        self.last_cycle = now;
        self.update_accumulator = self.update_accumulator.min(self.seconds_per_update * 2);
        self.render_accumulator = self.render_accumulator.min(self.seconds_per_render * 2);

        // render and/or update
        if self.render_next {
            self.render_accumulator = self.render_accumulator - self.seconds_per_render;
            let new_last_render = Instant::now();
            game.render(self.since_render(), self.since_update());
            self.last_render = new_last_render;
            self.stats.render();
        }

        if self.update_next {
            self.update_accumulator = self.update_accumulator - self.seconds_per_update;

            // we do this before the update instead of after because our timing code tries
            //   to make sure update starts are evenly spaced
            self.last_update = Instant::now();
            
            game.update();
            self.stats.update();
        }
        
        // schedule next action
        if self.render_accumulator >= self.seconds_per_render && self.update_accumulator >= self.seconds_per_update {
            self.render_next = true;
            self.update_next = true;
            return ControlFlow::Poll;
        } else if self.render_accumulator >= self.seconds_per_render {
            self.render_next = true;
            self.update_next = false;
            return ControlFlow::Poll;
        } else if self.update_accumulator >= self.seconds_per_update {
            self.render_next = false;
            self.update_next = true;
            return ControlFlow::Poll;
        } else if self.seconds_per_render - self.render_accumulator < self.seconds_per_update - self.update_accumulator {
            self.render_next = true;
            self.update_next = false;
            return ControlFlow::WaitUntil(now + chrono::Duration::to_std(&(self.seconds_per_render - self.render_accumulator)).unwrap());
        } else if self.seconds_per_update - self.update_accumulator < self.seconds_per_render - self.render_accumulator {
            self.render_next = false;
            self.update_next = true;
            return ControlFlow::WaitUntil(now + chrono::Duration::to_std(&(self.seconds_per_update - self.update_accumulator)).unwrap());
        } else {
            self.render_next = true;
            self.update_next = true;
            return ControlFlow::WaitUntil(now + chrono::Duration::to_std(&(self.seconds_per_update - self.update_accumulator)).unwrap());
        }
    }

    pub fn since_update(&self) -> Duration {
        Instant::now() - self.last_update
    }

    pub fn since_render(&self) -> Duration {
        Instant::now() - self.last_render
    }
}