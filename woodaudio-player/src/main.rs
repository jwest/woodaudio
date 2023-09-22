use std::error::Error;
use std::fs;
use std::thread;
use std::fmt;
use std::fs::File;
use std::io::BufReader;
use std::time::Duration;
use rand::seq::IteratorRandom;

use app_dirs2::AppDataType;
use app_dirs2::AppInfo;
use app_dirs2::get_app_root;
use redis::{Commands, Connection};
use rodio::{Decoder, OutputStream, Sink};

extern crate redis;

const APP_INFO: AppInfo = AppInfo{name: "woodaudio", author: "jwest" };

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

fn like_track(track: &Track) -> Result<(), Box<dyn Error>> {
    let data_dir = get_app_root(AppDataType::UserData, &APP_INFO)?;
    fs::create_dir_all(data_dir.clone())?;
    let new_file_path = data_dir.join(&track.id);
    fs::copy(&track.file_name, &new_file_path)?;
    println!("'Liked' file was saved in: {:?}", &new_file_path);
    Ok(())
}

fn add_liked_track(connection: &mut Connection) -> Result<(), Box<dyn Error>> {
    let data_dir = get_app_root(AppDataType::UserData, &APP_INFO)?;

    let liked_files = fs::read_dir(data_dir)?;
    
    let random_track = liked_files
        .choose(&mut rand::thread_rng())
        .unwrap()?;

    let _ = connection.set::<Option<&str>, Option<&str>, String>(random_track.file_name().to_str(), random_track.path().to_str());

    println!("Random track added: {:?}", random_track.path().display());
    Ok(())
}

fn main() -> ! {
    let mut connection = connect_redis();
    let mut connection_ps = connect_redis();

    let mut pubsub = connection_ps.as_pubsub();
    pubsub.set_read_timeout(Some(Duration::from_secs(1))).expect("Error with duration");
    pubsub.subscribe("player:control").expect("Error with subscribe");

    let mut song_played = 1;

    loop {
        if song_played % 5 == 0 {
            println!("add bonus 'liked' track to playlist after 5 played tracks");
            let _ = add_liked_track(&mut connection);
        }

        let current_track = read_next_track(&mut connection);
        println!("Next readed track: {:?}", current_track);

        match current_track {
            Some(track) => {
                let (_stream, stream_handle) = OutputStream::try_default().unwrap();
                let audio_file = match File::open(&track.file_name) {
                    Ok(it) => it,
                    Err(err) => {
                        println!("Audio file '{:?}' not exists, try next...", err);
                        ack_track(&mut connection, track.id);
                        continue;
                    },
                };
                let file = BufReader::new(audio_file);
                let source_result = Decoder::new(file);
    
                let source = match source_result {
                    Ok(file) => file,
                    Err(_) => continue,
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
                            "PLAY_OR_NEXT" => if sink.is_paused() {
                                sink.play();
                            } else {
                                sink.clear();
                            },
                            "LIKE" => {
                                match like_track(&track) {
                                    Err(err) => println!("Error on 'like' command {:?}", err),
                                    _ => ()
                                };
                            }
                            _ => ()
                        });
                }
    
                song_played += 1;
                ack_track(&mut connection, track.id);
            },
            None => {
                match add_liked_track(&mut connection) {
                    Ok(_) => (),
                    Err(_) => thread::sleep(Duration::from_secs(2)),
                };
            }
        }
    }
}
