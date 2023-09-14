use redis::Commands;
use redis::streams::{StreamReadOptions, StreamReadReply};

use std::fs::File;
use std::io::BufReader;
use rodio::{Decoder, OutputStream, Sink};

extern crate redis;

fn main() {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut connection = client.get_connection().expect("Connection to redis fail");

    let opts = StreamReadOptions::default()
        .block(3000)
        .count(1);

    let mut last_id = "0".to_string();

    loop {
        let results: Option<StreamReadReply> = connection.xread_options(&["downloaded_playlist"], &[last_id.as_str()], &opts).unwrap();
        println!("{:?}", results);

        if let Some(reply) = results {

            for stream_key in reply.keys {
                println!("->> xread block: {}", stream_key.key);
                for stream_id in stream_key.ids {
                    println!("  ->> StreamId: {:?}", stream_id);
                    let id = stream_id.id.clone();
                    last_id = id;

                    let url : String = stream_id.get("file_name").unwrap();
                    print!("{:?}", url);

                    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
                    let file = BufReader::new(File::open(url).unwrap());
                    let source = Decoder::new(file).unwrap();
                    let sink = Sink::try_new(&stream_handle).unwrap();
                    sink.append(source);
                    sink.sleep_until_end();

                    let _ = connection.xdel::<&str, &str, String>("downloaded_playlist", &[last_id.as_str()]);
                }
            }
            println!();
        }
    }
}