import redis
import requests
import tempfile
import hashlib
import json
from pathlib import Path
from time import time,sleep

r = redis.Redis(host='localhost', port=6379, decode_responses=True)

session_file_type = 'flac'

TMP_PATH = tempfile.mkdtemp()

def main():
    last_id = 0
    sleep_ms = 5000
    while True:
        try:
            downloaded_playlist_len = r.xlen('downloaded_playlist')
            print('Downloaded playlist size: ' + str(downloaded_playlist_len))

            if (downloaded_playlist_len <= 3):
                resp = r.xread({'playlist': last_id}, count=1, block=sleep_ms)
                if resp:
                    key, messages = resp[0]
                    last_id, data = messages[0]
                    print("REDIS ID: ", last_id)
                    print("      --> ", data)

                    file_name = Path(TMP_PATH).joinpath(hashlib.md5(data['url'].encode()).hexdigest())
                    print('Audio file download started: ' + data['full_name'] + ' | ' + str(file_name))

                    try:
                        response = requests.get(data['url'])

                        with open(str(file_name), "wb") as audioFile:
                            audioFile.write(response.content)
                            print('Audio file download completed: ' + str(file_name))

                            r.xadd('downloaded_playlist', {"url": data['url'], "full_name": data['full_name'], "ts": time(), "file_name": str(file_name)})
                            r.xdel('playlist', last_id)
                    except requests.exceptions.HTTPError as err:
                        print(err)
            else:
                print('Waiting 3s')
                sleep(3)

        except ConnectionError as e:
            print("ERROR REDIS CONNECTION: {}".format(e))

if __name__ == '__main__':
    main()