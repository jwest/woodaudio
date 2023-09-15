import redis
from multiprocessing import Process
import subprocess

from playlist import main as playlist_main
from downloader import main as downloader_main

WOODAUDIO_PLAYER_BIN = "/home/[[USER]]/woodaudio/woodaudio-player/target/release/woodaudio-player"

r = redis.Redis(host='localhost', port=6379, decode_responses=True)

def main():
    playlist_thread = Process(target=playlist_main, args=())
    playlist_thread.start()

    downloader_thread = Process(target=downloader_main, args=())
    downloader_thread.start()

    
    subprocess.run([WOODAUDIO_PLAYER_BIN])

if __name__ == '__main__':
    main()