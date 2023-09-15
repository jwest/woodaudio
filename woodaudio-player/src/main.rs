use redis::{Commands, Connection};
use redis::streams::{StreamReadOptions, StreamReadReply};

use std::fs::File;
use std::io::BufReader;
use std::time::Duration;
use rodio::{Decoder, OutputStream, Sink};

extern crate redis;

fn connect_redis() -> Connection {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let connection = client.get_connection().expect("Connection fail to redis");
    connection
}

struct Queue<'a> {
    last_id: String,
    connection: &'a mut Connection
}

impl Queue<'_> {
    fn new(connection: &mut Connection) -> Queue {
        let a: String = String::from("0");
        Queue{
            connection,
            last_id: a,
        }
    }
    fn pull(&mut self) -> Option<StreamReadReply> {
        let opts = StreamReadOptions::default()
            .block(3000)
            .count(1);
        let results: Option<StreamReadReply> = self.connection.xread_options(&["downloaded_playlist"], &[self.last_id.as_str()], &opts).unwrap();
        results
    }
    fn ack(&mut self) {
        let _ = self.connection.xdel::<&str, &str, String>("downloaded_playlist", &[self.last_id.as_str()]);
    }
}

fn main() {
    let mut connection = connect_redis();
    let mut connection_ps = connect_redis();

    let mut pubsub = connection_ps.as_pubsub();
    pubsub.set_read_timeout(Some(Duration::from_secs(1))).expect("Error with duration");
    pubsub.subscribe("player:control").expect("Error with subscribe");

    let mut queue = Queue::new(&mut connection);

    loop {
        let results: Option<StreamReadReply> = queue.pull();
        println!("{:?}", results);

        if let Some(reply) = results {

            for stream_key in reply.keys {
                println!("->> xread block: {}", stream_key.key);
                for stream_id in stream_key.ids {
                    println!("  ->> StreamId: {:?}", stream_id);
                    let id = stream_id.id.clone();
                    queue.last_id = id;
                    
                    let url : String = stream_id.get("file_name").unwrap();
                    println!("{:?}", url);

                    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
                    let file = BufReader::new(File::open(url).unwrap());
                    let source_result = Decoder::new(file);
                    let source = match source_result {
                        Ok(file) => file,
                        Err(error) => {
                            println!("Error with decode file {:?}", error);
                            queue.ack();
                            break;
                        },
                    };

                    let sink = Sink::try_new(&stream_handle).unwrap();
                    sink.append(source);
                    sink.play();

                    loop  {
                        print!(".");
                        if sink.empty() {
                            println!("Track ended...");
                            break;
                        }
                        let msg = pubsub.get_message();
                        if msg.is_ok() {
                            let unwrap_msg = msg.unwrap();
                            let payload : String = unwrap_msg.get_payload().unwrap();
                            println!("channel '{}': {}", unwrap_msg.get_channel_name(), payload);

                            if payload == "PLAY" {
                                sink.play();
                            } else if payload == "PAUSE" {
                                sink.pause();
                            } else if payload == "NEXT" {
                                sink.clear();
                                break;
                            }
                        }
                    }

                    // sink.sleep_until_end();

                    queue.ack();
                }
            }
            println!();
        }
    }
}

