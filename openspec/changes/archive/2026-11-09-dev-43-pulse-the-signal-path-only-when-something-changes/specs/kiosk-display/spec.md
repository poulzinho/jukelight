## MODIFIED Requirements

### Requirement: Wave motion
A pulse of light SHALL run slowly towards the end once when something
changes: when a track starts (also when the screen first shows it), when
the source or the route changes, when the quality changes, and when
playback resumes after a pause. Each arrival port SHALL light up as the
pulse arrives. While nothing changes the waves SHALL stand still; the
brightening wave and the ports still show the direction. The pulse and the port
SHALL move in about 15 steps a second rather than smoothly, so the browser
draws about 15 frames a second while a pulse runs instead of 60. With a route, the pulse
SHALL run along the first wave and, as it arrives, go on along the second,
so the light travels from the source to the sound box.
Between pulses no animation SHALL run in the signal path, so the browser
can stop drawing frames. The motion SHALL use only transform and opacity. With reduced motion, while paused, and on an
unavailable route, the waves SHALL stand still, with no pulses.

#### Scenario: Playing
- **WHEN** a track starts playing
- **THEN** one pulse moves towards the sound box

#### Scenario: Nothing changes
- **WHEN** a track keeps playing and nothing about it changes for a minute after its pulse
- **THEN** no further pulse runs

#### Scenario: Quality changes
- **WHEN** the quality of the playing track changes
- **THEN** one pulse runs

#### Scenario: Playback resumes
- **WHEN** playback resumes after a pause
- **THEN** one pulse runs

#### Scenario: From one end to the other
- **WHEN** a track plays through Music Assistant
- **THEN** the pulse on the second wave starts as the pulse on the first wave arrives

#### Scenario: Stepped pulse
- **WHEN** a pulse runs along a wave for 2.2 seconds
- **THEN** it moves in 33 steps, and so does its arrival port

#### Scenario: Between pulses
- **WHEN** a pulse has reached the end
- **THEN** no animation runs in the signal path

#### Scenario: Reduced motion
- **WHEN** the browser prefers reduced motion
- **THEN** no pulse moves

#### Scenario: Paused
- **WHEN** the player pauses
- **THEN** no pulse moves
