use std::{thread, time::Duration};
use std::error::Error;
use log::info;
use rppal::gpio::Level;
use crate::Config;
use crate::state::{PlayerBus, Message};

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

        let next_song_pin = gpio.get(self.config.gpio.next_song_pin.try_into()?)?.into_input_pullup();
        let like_song_pin = gpio.get(self.config.gpio.like_song_pin.try_into()?)?.into_input_pullup();
        let suspend_pin = gpio.get(self.config.gpio.suspend_pin.try_into()?)?.into_input_pullup();

        let bus_next = self.player_bus.clone();
        let bus_like = self.player_bus.clone();
        let bus_suspend = self.player_bus.clone();

        loop {
            if next_song_pin.read() == Level::Low {
                thread::sleep(Duration::from_millis(50));
                if next_song_pin.read() == Level::Low {
                    info!("Next song button pressed");
                    bus_next.publish_message(Message::UserPlayNext);
                    while next_song_pin.read() == Level::Low {
                        thread::sleep(Duration::from_millis(10));
                    }
                }
            }

            if like_song_pin.read() == Level::Low {
                thread::sleep(Duration::from_millis(50));
                if like_song_pin.read() == Level::Low {
                    info!("Like song button pressed");
                    bus_like.publish_message(Message::UserLike);
                    while like_song_pin.read() == Level::Low {
                        thread::sleep(Duration::from_millis(10));
                    }
                }
            }

            if suspend_pin.read() == Level::Low {
                thread::sleep(Duration::from_millis(50));
                if suspend_pin.read() == Level::Low {
                    info!("Suspend button pressed");
                    bus_suspend.publish_message(Message::UserSuspendToggle);
                    while suspend_pin.read() == Level::Low {
                        thread::sleep(Duration::from_millis(10));
                    }
                }
            }

            thread::sleep(Duration::from_millis(10));
        }
    }
}