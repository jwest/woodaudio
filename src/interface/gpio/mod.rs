use std::{thread, time::Duration};

use log::info;

use crate::{config::{self, Config}, state::PlayerBus};

pub struct Gpio {
    config: Config,
    player_bus: PlayerBus,
}

impl Gpio {
    pub fn new(config: Config, player_bus: PlayerBus) -> Self {
        Gpio { config, player_bus }
    }

    pub fn wait(&self) {
        let gpio = rppal::gpio::Gpio::new()?;
        let mut next_song_pin = gpio.get(self.config.gpio.next_song_pin)?.into_input_pulldown();
        let mut like_song_pin = gpio.get(self.config.gpio.like_song_pin)?.into_input_pulldown();
        
        next_song_pin.set_async_interrupt(
            Trigger::FallingEdge,
            Some(Duration::from_millis(50)),
            move |event| {
                info!("Next song button pressed, event = {:?}", event);
                self.player_bus.publish_command(crate::state::Command::Next);
            },
        )?;

        like_song_pin.set_async_interrupt(
            Trigger::FallingEdge,
            Some(Duration::from_millis(50)),
            move |event| {
                info!("Next song button pressed, event = {:?}", event);
                self.player_bus.publish_command(crate::state::Command::Like(()));
                self.player_bus.publish_command(crate::state::Command::Radio(()));
            },
        )?;
    }
}