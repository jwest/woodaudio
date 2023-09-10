import redis
import tidalapi
import configparser
import random
import requests
from time import time

session_audio_quality = tidalapi.Quality.hi_res_lossless

r = redis.Redis(host='localhost', port=6379, decode_responses=True)

def main():
    config_file_name = 'config.ini'
    config = configparser.ConfigParser()
    config.read(config_file_name)

    session = tidalapi.Session()

    def tidal_login():
        session.login_oauth_simple()
        config['tidal'] = {}
        config['tidal']['token_type'] = session.token_type
        config['tidal']['access_token'] = session.access_token
        config['tidal']['refresh_token'] = session.refresh_token
        config['tidal']['expiry_time'] = str(session.expiry_time)
        with open(config_file_name, 'w') as configfile:
            config.write(configfile)

    if ('tidal' not in config or not config['tidal']['token_type']):
        tidal_login()
        print('auth saved in config file')
    else:
        try:
            session.load_oauth_session(
                 config['tidal']['token_type'],
                 config['tidal']['access_token'],
                 config['tidal']['refresh_token'],
                 config['tidal']['expiry_time']
            )
            print('session restored from config file')
        except Exception as err:
            print(err)
            tidal_login()

    session.audio_quality = session_audio_quality

    home = session.home()

    PLAYLIST = []

    try:
        for category in home.categories:
            for item in category.items:
                if isinstance(item, tidalapi.Mix):
                    for track in item.items():
                        print (track.get_url())
                        PLAYLIST.append(track)
                        break
                    break

                if (len(PLAYLIST) >= 2):
                    break

    except requests.exceptions.HTTPError as err:
        print(err)

    print('PLAYLIST: ' + str(len(PLAYLIST)))
    print(PLAYLIST)
    EXTEND_PLAYLIST = []

    for track in PLAYLIST:
        try:
            EXTEND_PLAYLIST.append(track)
            for p in track.get_track_radio(5):
                EXTEND_PLAYLIST.append(p)
        except requests.exceptions.HTTPError as err:
            print(err)
            time.sleep(2)

    # PLAYLIST = list(map(lambda t: Track(t), EXTEND_PLAYLIST))
    print('EXTEND_PLAYLIST: ' + str(len(EXTEND_PLAYLIST)))

    # PLAYLIST = list(map(lambda t: Track(t), PLAYLIST))
    PLAYLIST = EXTEND_PLAYLIST

    random.shuffle(PLAYLIST)

    for track in PLAYLIST:
        r.xadd('playlist', {"url": track.get_url(), "full_name": track.full_name, "ts": time()})
    print("playlist length: " + str(r.xlen('playlist')))

if __name__ == '__main__':
    main()

