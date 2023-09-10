import redis
from time import sleep
from multiprocessing import Process

from playlist import main as playlist_main
from downloader import main as downloader_main
from player import main as player_main

r = redis.Redis(host='localhost', port=6379, decode_responses=True)

def main():
    playlist_thread = Process(target=playlist_main, args=())
    playlist_thread.start()

    downloader_thread = Process(target=downloader_main, args=())
    downloader_thread.start()

    player_thread = Process(target=player_main, args=())
    player_thread.start()

    # while True:
        # r.publish('player:next-track', 'message')

if __name__ == '__main__':
    main()