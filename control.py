#!/usr/bin/env python3
import redis
import RPi.GPIO as GPIO
import time

r = redis.Redis(host='localhost', port=6379, decode_responses=True)

GPIO.setmode(GPIO.BCM)
GPIO.setup(23, GPIO.IN, pull_up_down=GPIO.PUD_UP)
GPIO.setup(24, GPIO.IN, pull_up_down=GPIO.PUD_UP)

def like(_):
  print('"LIKE" command send to redis')
  r.publish('player:control', 'LIKE')

def play_or_next(_):
  print('"PLAY_OR_NEXT" command send to redis')
  r.publish('player:control', 'PLAY_OR_NEXT')

GPIO.add_event_detect(23, GPIO.FALLING, callback=like, bouncetime=300)
GPIO.add_event_detect(24, GPIO.FALLING, callback=play_or_next, bouncetime=300)

try:
    while True : time.sleep(0.2)
except:
    GPIO.cleanup()
