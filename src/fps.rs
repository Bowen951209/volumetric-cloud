#[derive(Default)]
pub struct FpsCounter {
    time: Option<std::time::Instant>,
    fps: Option<f32>,
}

impl FpsCounter {
    pub fn start_frame(&mut self) {
        if let Some(last_time) = self.time {
            self.fps = Some(1.0 / last_time.elapsed().as_secs_f32());
        }
        self.time = Some(std::time::Instant::now());
    }

    pub fn fps(&self) -> Option<f32> {
        self.fps
    }
}
