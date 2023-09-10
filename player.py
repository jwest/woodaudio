import redis
import subprocess
from time import time

r = redis.Redis(host='localhost', port=6379, decode_responses=True)
p = r.pubsub()
p.subscribe('jou')

session_file_type = 'flac'

def main():
    last_id = 0
    sleep_ms = 5000
    while True:
        try:
            resp = r.xread({'downloaded_playlist': last_id}, count=1, block=sleep_ms)
            if resp:
                key, messages = resp[0]
                last_id, data = messages[0]
                print("REDIS ID: ", last_id)
                print("      --> ", data)

                try:
                    print('Played: ' + data['file_name'])

                    r.xadd('archive', {"url": data['url'], "full_name": data['full_name'], "ts": time(), "file_name": data['file_name']})
                    r.xdel('downloaded_playlist', last_id)

                    player_process = subprocess.Popen(['ffplay', '-nodisp', '-autoexit', data['file_name']])
                    while player_process.poll() is None:
                        message = p.get_message()
                        if (message is not None and message['type'] == 'message'):
                            player_process.terminate()

                except Exception as err:
                    print(err)

        except ConnectionError as e:
            print("ERROR REDIS CONNECTION: {}".format(e))

if __name__ == '__main__':
    main()