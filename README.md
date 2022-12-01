# Micro VSRG
A rhythm game for Raspberry Pi - specifically the Raspberry Pi 3B+. Imports 4k osu!mania maps (no hold notes). Single player (2 player may be added). I'm done with this project since this was for a school assignment. Any further updates will most likely be done through an outside PR. This game is meant to be played with wheeled robots hitting the notes and sacrifices were made for that. However, it can be easily modified.

# Showcase
https://user-images.githubusercontent.com/16981283/204935130-90d533c1-e841-42fe-a69b-7510cf3262cf.mp4

# Instructions
1. Put the osu!mania map sets you want to use in the map_depot folder in the execution directory.
2. Plug in buttons (B1 to GPIO 4, B2 to GPIO 17, B3 to GPIO 22, and B4 to GPIO 9) - configured in input_pulldown (3.3v to the input pin to trigger).
3. Plug in leds (LED1 to GPIO 2, LED2 to GPIO 3, LED3 to GPIO 27, and LED4 to GPIO 10).
4. Run the program.
5. Select map.
6. Game begins.
7. Whenever, an led is lit up, press the corresponding button to get the highest accuracy.

# Notes
- The game is not perfect. It's not perfect at all. (thank you Copilot)
- Holding buttons counts as a hit.
