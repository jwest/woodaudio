import multiprocessing
import tidalapi
from pathlib import Path
from pydub import AudioSegment
from pydub.playback import play
import requests
import time
import tempfile
import configparser
import hashlib
import random

# Override the required playback quality, if necessary
# Note: Set the quality according to your subscription.
# Normal: Quality.low_320k
# HiFi: Quality.high_lossless
# HiFi+ Quality.hi_res_lossless
session_audio_quality = tidalapi.Quality.hi_res_lossless
session_file_type = 'flac'

TMP_PATH = tempfile.mkdtemp()

class Track:
    def __init__(self, track):
        self.track = track
        self.file_name = self._generate_file_name()
        self.preload_lock_file_name = Path(str(self.file_name) + '_lock')

    def play(self, _ = ""):
        if (not self.is_preload()):
            return
        try:
            audioSegmentFile = AudioSegment.from_file(str(self.file_name), session_file_type)
            audioFragment = audioSegmentFile[:15000]
            play(audioFragment)
        except(err):
            print(err)

    def preload(self, _ = ""):
        if (self.is_preload() or self.is_preloaded()):
            return

        print('Audio file download started: ' + str(self.file_name))
        self._put_preload_lock()
        try:
            response = requests.get(self.track.get_url())

            with open(str(self.file_name), "wb") as audioFile:
                audioFile.write(response.content)
                print('Audio file download completed: ' + str(self.file_name))
        except requests.exceptions.HTTPError as err:
            print(err)
            time.sleep(10)
            return self.preload()
    def is_preload(self):
        return self.preload_lock_file_name.exists()

    def is_preloaded(self):
        return self.file_name.exists()

    def _put_preload_lock(self):
        self.preload_lock_file_name.touch()

    def _generate_file_name(self):
        return Path(TMP_PATH).joinpath(hashlib.md5(self.track.get_url().encode()).hexdigest())

    def __str__ (self):
        return 'Track(track=' + self.track.get_url() + ')'

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
        except:
            print('error')
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
            EXTEND_PLAYLIST.append(Track(track))
            for p in track.get_track_radio(5):
                EXTEND_PLAYLIST.append(Track(p))
        except requests.exceptions.HTTPError as err:
            print(err)
            time.sleep(2)

    # PLAYLIST = list(map(lambda t: Track(t), EXTEND_PLAYLIST))
    print('EXTEND_PLAYLIST: ' + str(len(EXTEND_PLAYLIST)))

    # PLAYLIST = list(map(lambda t: Track(t), PLAYLIST))
    PLAYLIST = EXTEND_PLAYLIST

    random.shuffle(PLAYLIST)
    index = 0
    play_track_process = ''

    while True:

        if (index >= len(PLAYLIST)):
            break

        current_track = PLAYLIST[index]
        if (not current_track.is_preload()):
            preload_current_track_process = multiprocessing.Process(target=current_track.preload, args=(current_track,))
            preload_current_track_process.start()

        if (current_track.is_preloaded()):

            if (index + 1 <= len(PLAYLIST) and not PLAYLIST[index + 1].is_preload()):
                next_track = PLAYLIST[index + 1]
                preload_next_track_process = multiprocessing.Process(target=next_track.preload, args=(next_track,))
                preload_next_track_process.start()

            if (not play_track_process or not play_track_process.is_alive()):
                play_track_process = multiprocessing.Process(target=current_track.play, args=(current_track,))
                play_track_process.start()
                index = index + 1

        else:
            time.sleep(1)

if __name__ == '__main__':
    multiprocessing.freeze_support()
    main()





