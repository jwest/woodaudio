import redis
import tidalapi
import configparser
import random
import requests
import tempfile
import hashlib
from pathlib import Path
from time import time,sleep

CONFIG_FILE_NAME = 'config.ini'
TMP_PATH = tempfile.mkdtemp()
SLEEP_MS = 5000
r = redis.Redis(host='localhost', port=6379, decode_responses=True)

def wait_for_internet_connection():
    while True:
        try:
            requests.head('https://api.tidal.com', timeout=2)
        except requests.exceptions.HTTPError as err:
            print('Waiting for internet connection' + str(err))
            sleep(2)
        except requests.exceptions.ConnectionError as err:
            print('Waiting for internet connection')
            sleep(2)
        else:
            print('Internet connection present')
            return

def setup_session():
    try:
        config = configparser.ConfigParser()
        config.read(CONFIG_FILE_NAME)

        session = tidalapi.Session()

        def tidal_login():
            session.login_oauth_simple()
            config['tidal'] = {}
            config['tidal']['audio_quality'] = 'HI_RES_LOSSLESS'
            config['tidal']['token_type'] = session.token_type
            config['tidal']['access_token'] = session.access_token
            config['tidal']['refresh_token'] = session.refresh_token
            config['tidal']['expiry_time'] = str(session.expiry_time)
            with open(CONFIG_FILE_NAME, 'w') as configfile:
                config.write(configfile)
                print('auth saved in config file')

        if ('tidal' not in config or not config['tidal']['token_type']):
            tidal_login()
        else:
            try:
                while not session.load_oauth_session(
                    config['tidal']['token_type'],
                    config['tidal']['access_token'],
                    config['tidal']['refresh_token'],
                    config['tidal']['expiry_time']
                ):
                    print('Login to tidal failed')
                print('session restored from config file')
            except Exception as err:
                print(err)
                tidal_login()

        session.audio_quality = tidalapi.Quality.hi_res_lossless
        return session
    except Exception as err:
        print(err)
        sleep(3)
        setup_session()
    
def waiting():
    total_keys = r.dbsize()
    print('All keys: ' + str(total_keys))
    while total_keys >= 10:
        sleep(5)

def download_track(track):
    waiting()
    track_id = str(hashlib.md5(track.get_url().encode()).hexdigest())
    file_name = Path(TMP_PATH).joinpath(track_id)
    try:
        track_url = track.get_url()
        print('Download track: ' + track_url + ' as file: ' + str(file_name))
        response = requests.get(track_url)
        with open(str(file_name), "wb") as audioFile:
            audioFile.write(response.content)
            print('Downloaded track: ' + track_url + ' as file saved: ' + str(file_name))
            return {
                'id': track_id,
                'file_name': file_name,
            }

    except requests.exceptions.HTTPError as err:
        print(err)
        sleep(3)
        return download_track(track)

def main():
    wait_for_internet_connection()
    session = setup_session()

    try:
        for_you_response = session.for_you();
        for category in for_you_response.categories:
            for item in category.items:
                if isinstance(item, tidalapi.Mix) or isinstance(item, tidalapi.Album) or isinstance(item, tidalapi.Playlist):
                    if isinstance(item, tidalapi.Mix):
                        print('Discover ' + str(item) + ' item: ' + item.title)
                    else:
                        print('Discover ' + str(item) + ' item: ' + item.name)
                    for track in item.items():
                        downloaded_track = download_track(track)
                        print(downloaded_track)
                        r.set(downloaded_track['id'], str(downloaded_track['file_name']))
                        sleep(2)

    except requests.exceptions.HTTPError as err:
        print(err)

if __name__ == '__main__':
    main()
