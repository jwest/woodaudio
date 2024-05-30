use crate::state::{Message, PlayerBus};

pub struct Systray {
    playerbus: PlayerBus,
    sysbar: sysbar::Sysbar,
}

impl Systray {
    pub fn init(playerbus: PlayerBus) -> Self {
        Self { 
            playerbus,
            sysbar: sysbar::Sysbar::new("Woodaudio"),
        }
    }

    pub fn display(&mut self) {
        self.add_item("Play", Message::UserPlay);
        self.add_item("Pause", Message::UserPause);
        self.add_item("Next", Message::UserPlayNext);
        self.add_item("Like Track", Message::UserLike);
        self.add_item("Track Radio", Message::UserLoadRadio);
        self.sysbar.add_quit_item("Quit");
        self.sysbar.display();
    }

    fn add_item(&mut self, label: &str, message: Message) {
        let playerbus = self.playerbus.clone();
        self.sysbar.add_item(
            label,
            Box::new(move || {
                playerbus.publish_message(message.clone());
            }),
        );
    }
}