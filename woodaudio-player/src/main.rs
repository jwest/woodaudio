use std::thread;
use std::fmt;
use std::fs::File;
use std::io::BufReader;
use std::time::Duration;

use redis::{Commands, Connection};
use rodio::{Decoder, OutputStream, Sink};

extern crate redis;

fn connect_redis() -> Connection {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let connection = client.get_connection().expect("Connection fail to redis");
    connection
}

#[derive(Debug, Clone)]
struct Track {
    id: String,
    file_name: String,
}

impl fmt::Display for Track {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Track")
            .field("id", &self.id)
            .field("file_name", &self.file_name)
            .finish()
    }
}

fn read_next_track(connection: &mut Connection) -> Option<Track> {
    return match redis::cmd("RANDOMKEY").query::<String>(connection) {
        Ok(track_id) => match connection.get(track_id.clone()) {
                Ok(track_file_name) => Some(Track { id: track_id, file_name: track_file_name }),
                Err(_) => None
        },
        Err(_) => None,
    };
}

fn ack_track(connection: &mut Connection, track_id: String) {
    redis::cmd("DEL").arg(track_id).execute(connection);
}

fn main() {
    let mut connection = connect_redis();
    let mut connection_ps = connect_redis();

    let mut pubsub = connection_ps.as_pubsub();
    pubsub.set_read_timeout(Some(Duration::from_secs(1))).expect("Error with duration");
    pubsub.subscribe("player:control").expect("Error with subscribe");

    loop {
        let current_track = read_next_track(&mut connection);
        println!("Next readed track: {:?}", current_track);

        match current_track {
            Some(track) => {
                let (_stream, stream_handle) = OutputStream::try_default().unwrap();
                let file = BufReader::new(File::open(track.file_name).unwrap());
                let source_result = Decoder::new(file);
    
                let source = match source_result {
                    Ok(file) => file,
                    Err(_) => return,
                };
    
                let sink = Sink::try_new(&stream_handle).unwrap();
                sink.append(source);
                sink.play();
    
                loop  {
                    if sink.empty() {
                        println!("Playing track ended.");
                        break;
                    }
                    let _ = pubsub.get_message()
                        .map(|msg| msg.get_payload::<String>())
                        .map(|payload| payload.unwrap())
                        .map(|command| match command.as_str() {
                            "PLAY" => sink.play(),
                            "PAUSE" => sink.pause(),
                            "NEXT" => sink.clear(),
                            _ => {}
                        });
                }
    
                ack_track(&mut connection, track.id);
            },
            None => thread::sleep(Duration::from_secs(2)),
        }
    }
}
