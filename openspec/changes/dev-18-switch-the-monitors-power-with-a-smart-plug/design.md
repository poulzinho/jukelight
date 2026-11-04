## Context

The display agent (`host/display-agent/jukelight-display-agent`, POSIX sh)
follows the backend's decisions and switches the outputs with `swaymsg`.
It runs as a systemd user service of the kiosk user and reads
`/etc/jukelight/host.env` (`JUKELIGHT_URL`), which any user can read.

On the reference host the monitor hangs off a wireless HDMI extender whose
receiver shows a logo when the output is off, so the monitor never sleeps.
A Shelly Plug M Gen3 now powers the monitor. Measured by hand on the
reference host:

- `Switch.Set?id=0&on=false|true` over HTTP with digest authentication;
- monitor 35.6 W when on, 0.0 W when the plug is off;
- picture back about 2 s after switching on, with the extender's logo
  briefly while the receiver picks up the signal;
- `initial_state` set to `on` through `Switch.SetConfig`, because the
  plug's web page doesn't offer it, so the monitor comes back after a power
  cut.

## Goals / Non-Goals

**Goals:** the monitor really switches off and on with the decisions, on
setups where switching the output is not enough; setups without a plug
behave exactly as before; the plug's password never ends up anywhere other
users can read it.

**Non-Goals:** a plug client in the backend; per-brand plug support in the
agent; CEC or DDC/CI.

## Decisions

### 1. Plain commands, run after the output switch

Two settings, `POWER_ON_COMMAND` and `POWER_OFF_COMMAND`, run with `sh -c`
after `swaymsg`. Switching off: outputs off, then the plug off. Switching
on: outputs on, then the plug on, so the picture is already there when the
monitor wakes. Unset, nothing extra runs.

- Alternative: the plug instead of `swaymsg`. Rejected: with the output
  left on, the extender would show the picture right away, but on setups
  without an extender, blanking the output is what saves power; keeping
  both costs nothing.
- Alternative: a Shelly client in the agent (address and password as
  settings). Rejected: it ties the agent to one brand; a command works with
  any plug, relay or script.

### 2. A failing command is retried in the background

The agent runs the command in a background job that retries until it
succeeds: first after the retry interval, then after twice the previous
wait, up to 5 minutes. A fixed interval kept the reference host's plug
locked (found during the acceptance: after a few refused passwords it
answers HTTP 429 and accepts nothing for minutes, and a retry every 5 s
renewed the lock); growing waits let the lock expire while still reacting
quickly to a short network glitch. A new decision stops the running job (and its
sleep, as the watchdog does) before starting its own, so an old "on" never
lands after a newer "off". The first failure and the recovery are logged,
not every attempt, so an unplugged plug doesn't flood the journal. The job
doesn't touch the failed-switches count: restarting the agent doesn't fix
a plug that is unreachable.

- Alternative: log only, wait for the next decision. Rejected: the monitor
  could stay dark while music plays.
- Alternative: run the command in the foreground. Rejected: a plug that
  doesn't answer would hold up reading the decision stream (and the
  silence limit).

### 3. Private settings for the kiosk user

The unit reads a second, optional settings file:
`EnvironmentFile=-%h/.config/jukelight/display-agent.env`, owned by the
kiosk user with mode 600. The power commands go there. The plug's
password goes into a curl credentials file (`netrc`) next to it, also mode
600, so it appears neither in the commands nor in the process list:

```
POWER_ON_COMMAND=curl -fsS --max-time 5 --digest --netrc-file /home/kiosk/.config/jukelight/plug.netrc "http://192.0.2.10/rpc/Switch.Set?id=0&on=true"
POWER_OFF_COMMAND=curl -fsS --max-time 5 --digest --netrc-file /home/kiosk/.config/jukelight/plug.netrc "http://192.0.2.10/rpc/Switch.Set?id=0&on=false"
```

`-f` makes a wrong password (HTTP 401) a failure, so it is retried and
logged.

The plug's password should be letters and digits only (found during the
acceptance): in `plug.netrc` quotes and backslashes need escaping, and a
password with a `'` was refused there although it worked at curl's prompt.
After a few refused attempts the plug answers HTTP 429 and accepts nothing
for several minutes, so a wrong password must be fixed with the agent
stopped.

- Alternative: `/etc/jukelight/host.env`. Rejected: readable by every user,
  and also read by the kiosk session.
- Alternative: the password in the command (`-u admin:…`). Rejected: it
  would show in the process list and in `systemctl show`.

### 4. The plug's own setup

Reference setup only: fixed address (a DHCP reservation), current firmware,
authentication on, cloud, Matter, Bluetooth and its own access point off,
`initial_state` `on` through `Switch.SetConfig`. Matter off, because any
paired controller could otherwise switch the monitor behind Jukelight's
back.

## Risks / Trade-offs

- [The plug is unreachable when the music starts] → the agent retries
  every 5 s; the output is already on, so the picture appears as soon as
  the plug answers.
- [Someone switches the plug by hand] → the next decision switches it back.
- [The monitor needs longer than 2 s on another setup] → the picture comes
  when it is ready; nothing in the agent waits for it.

## Tests

Agent tests in `crates/app/tests/display_agent.rs` (the agent is tested as
a process against a fake backend, with a fake `swaymsg`). The power
commands are fake commands that append to the same call log as `swaymsg`,
so the order shows:

- "Switch on with a power-on command", "Switch off with a power-off
  command" and "No power commands" (the existing tests, unchanged).
- "Plug unreachable for a while": a command that fails twice, then
  succeeds; three calls, the agent still running.
- "Decision changes while retrying": a failing on command, then the
  decision off; the on command stops and the off command runs.
- "Backend unreachable with a power-off command".
- All agent tests under bash and dash.

On the reference host: wake time, 0 W when off, the fail-safe switching the
plug off, and the plug's status `source` showing the agent's switches.
