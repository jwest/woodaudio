use std::{thread, time::Duration};
use std::error::Error;
use log::info;
use rppal::gpio::Trigger;
use crate::{config::{self, Config}, state::PlayerBus};
use crate::state::Message;

pub struct Gpio {
    config: Config,
    player_bus: PlayerBus,
}

impl Gpio {
    pub fn new(config: Config, player_bus: PlayerBus) -> Self {
        Self { config, player_bus }
    }

    pub fn wait(&self) -> Result<(), Box<dyn Error>> {
        let gpio = rppal::gpio::Gpio::new()?;
        let mut next_song_pin = gpio.get(self.config.gpio.next_song_pin.try_into()?)?.into_input_pullup();
        let mut like_song_pin = gpio.get(self.config.gpio.like_song_pin.try_into()?)?.into_input_pullup();

        let player_bus_next_song = self.player_bus.clone();
        let player_bus_like_song = self.player_bus.clone();

        next_song_pin.set_async_interrupt(
            Trigger::FallingEdge,
            Some(Duration::from_millis(50)),
            move |event| {
                info!("Next song button pressed, event = {:?}", event);
                player_bus_next_song.publish_message(Message::UserPlayNext);
            },
        )?;

        like_song_pin.set_async_interrupt(
            Trigger::FallingEdge,
            Some(Duration::from_millis(50)),
            move |event| {
                info!("Like song button pressed, event = {:?}", event);
                player_bus_like_song.publish_message(Message::UserLike);
            },
        )?;

        loop {
            thread::sleep(Duration::from_secs(1));
        }
    }
}