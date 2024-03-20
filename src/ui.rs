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
    _button_a: Button,
    _button_b: Button,
    state: UiState,
}

impl Ui {
    /// Constructs Ui.
    pub fn new(knob: Knob, _button_a: Button, _button_b: Button) -> Self {
        Self {
            knob,
            _button_a,
            _button_b,
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
            for led in 0..3 {
                if level != self.state.levels[led] {
                    self.state.levels[led] = level;
                    self.state.show();
                    set_rgb_levels(|rgb| {
                        *rgb = self.state.levels;
                    })
                    .await;
                }
            }
            Timer::after_millis(50).await;
        }
    }
}
