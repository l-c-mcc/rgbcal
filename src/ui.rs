use crate::*;

struct UiState {
    levels: [u32; 3],
    frame_rate: u64,
}

impl UiState {
    /// Prints current frame rate and RGB levels.
    fn show(&self) {
        let names = ["red", "green", "blue"];
        rprintln!();
        for (name, level) in names.iter().zip(self.levels.iter()) {
            rprintln!("{}: {}", name, level);
        }
        rprintln!("frame rate: {}", self.frame_rate);
    }
}

impl Default for UiState {
    /// Default values for UiState. Max RGB brightness and frame rate at
    /// two thirds of the max value.
    fn default() -> Self {
        Self {
            levels: [LEVELS - 1, LEVELS - 1, LEVELS - 1],
            frame_rate: 100,
        }
    }
}

pub struct Ui {
    knob: Knob,
    button_a: Button,
    button_b: Button,
    state: UiState,
}

impl Ui {
    /// Constructs Ui.
    pub fn new(knob: Knob, button_a: Button, button_b: Button) -> Self {
        Self {
            knob,
            button_a,
            button_b,
            state: UiState::default(),
        }
    }

    /// Read pontentiometer and buttons and updates the RGB
    /// global accordingly.
    pub async fn run(&mut self) -> ! {
        let mut level = self.knob.measure().await;
        for led in 0..3 {
            self.state.levels[led] = level;
        }
        set_rgb_levels(|rgb| {
            *rgb = self.state.levels;
        })
        .await;
        self.state.show();
        loop {
            level = self.knob.measure().await;
            match(self.button_a.is_low(),self.button_b.is_low()) {
                (true , true ) => self.update_led(level, RED).await,
                (true , false) => self.update_led(level, BLUE).await,
                (false, true ) => self.update_led(level, GREEN).await,
                (false, false) => {},
            }
            
            Timer::after_millis(50).await;
        }
    }

    async fn update_led(&mut self, level: u32, led: usize) {
        if level != self.state.levels[led] {
            self.state.levels[led] = level;
            self.state.show();
            set_rgb_levels(|rgb| {
                *rgb = self.state.levels;
            })
            .await;
        }
    }
}
