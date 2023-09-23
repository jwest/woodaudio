#!/usr/bin/env python3
import redis
import RPi.GPIO as GPIO
import time

r = redis.Redis(host='localhost', port=6379, decode_responses=True)

GPIO.setmode(GPIO.BCM)

GPIO.setup(4, GPIO.IN, pull_up_down=GPIO.PUD_UP)
GPIO.setup(23, GPIO.IN, pull_up_down=GPIO.PUD_UP)
GPIO.setup(24, GPIO.IN, pull_up_down=GPIO.PUD_UP)

while True:
    input_state1 = GPIO.input(4)
    input_state2 = GPIO.input(23)
    input_state3 = GPIO.input(24)
    if input_state1 == False:
        print('Button 1 Pressed')
        r.publish('player:control', 'PLAY_OR_PAUSE')
        time.sleep(0.2)
    if input_state2 == False:
        print('Button 2 Pressed')
        r.publish('player:control', 'LIKE')
        time.sleep(0.2)
    if input_state3 == False:
        print('Button 3 Pressed')
        r.publish('player:control', 'PLAY_OR_NEXT')
        time.sleep(0.2)
