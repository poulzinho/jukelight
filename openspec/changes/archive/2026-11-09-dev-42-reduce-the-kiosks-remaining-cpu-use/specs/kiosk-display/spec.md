## MODIFIED Requirements

### Requirement: Wave motion
A pulse of light SHALL run slowly towards the end every 8 seconds, and each
arrival port SHALL light up as the pulse arrives. The pulse and the port
SHALL move in about 15 steps a second rather than smoothly, so the browser
draws about 15 frames a second while a pulse runs instead of 60. With a route, the pulse
SHALL run along the first wave and, as it arrives, go on along the second,
so the light travels from the source to the sound box.
Between pulses no animation SHALL run in the signal path, so the browser
can stop drawing frames. The motion SHALL use only transform and opacity. With reduced motion, while paused, and on an
unavailable route, the waves SHALL stand still, with no pulses.

#### Scenario: Playing
- **WHEN** a track plays
- **THEN** the pulses move towards the sound box

#### Scenario: From one end to the other
- **WHEN** a track plays through Music Assistant
- **THEN** the pulse on the second wave starts as the pulse on the first wave arrives

#### Scenario: Stepped pulse
- **WHEN** a pulse runs along a wave for 2.2 seconds
- **THEN** it moves in 33 steps, and so does its arrival port

#### Scenario: Between pulses
- **WHEN** a pulse has reached the end and the next one is not due yet
- **THEN** no animation runs in the signal path

#### Scenario: Reduced motion
- **WHEN** the browser prefers reduced motion
- **THEN** no pulse moves

#### Scenario: Paused
- **WHEN** the player pauses
- **THEN** no pulse moves
