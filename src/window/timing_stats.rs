use std::time::Duration;
use web_time::Instant;



pub struct TimingStats {
    ups_count: u32,
    fps_count: u32,
    last_timing_block_start: Instant,
    timing_block_duration: Duration,
    last_ups: f32,
    last_fps: f32,
}

impl TimingStats {
    pub fn ups(&self) -> f32 {
        self.last_ups
    }

    pub fn fps(&self) -> f32 {
        self.last_fps
    }

    pub fn new() -> TimingStats {
        TimingStats {
            ups_count: 0,
            fps_count: 0,
            last_timing_block_start: Instant::now(),
            timing_block_duration: Duration::from_secs_f32(20.0),
            last_ups: 0.0,
            last_fps: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.ups_count = 0;
        self.fps_count = 0;
        self.last_timing_block_start = Instant::now();
        self.last_ups = 0.0;
        self.last_fps = 0.0;
    }

    pub fn update(&mut self) {
        self.ups_count += 1;
        self.check_for_time_block_end();
    }

    pub fn render(&mut self) {
        self.fps_count += 1;
        self.check_for_time_block_end();
    }

    fn check_for_time_block_end(&mut self) {
        let time_since_last = Instant::now() - self.last_timing_block_start;
        if time_since_last >= self.timing_block_duration {
            self.last_ups = self.ups_count as f32 / time_since_last.as_secs_f32();
            self.last_fps = self.fps_count  as f32 / time_since_last.as_secs_f32();

            self.ups_count = 0;
            self.fps_count = 0;
            self.last_timing_block_start = Instant::now();


            self.print_stats();
        }
    }

    fn print_stats(&self) {
        println!("fps: {0}", self.fps());
        println!("ups: {0}", self.ups());
    }
}