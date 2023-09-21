#!/usr/bin/env python3
import redis
from multiprocessing import Process
import subprocess

from playlist import main as playlist_main

WOODAUDIO_PLAYER_BIN = "/home/jwest/woodaudio/woodaudio-player/target/release/woodaudio-player"

r = redis.Redis(host='localhost', port=6379, decode_responses=True)
r.flushall()

def main():
    playlist_thread = Process(target=playlist_main, args=())
    playlist_thread.start()

    subprocess.run([WOODAUDIO_PLAYER_BIN])

if __name__ == '__main__':
    main()
