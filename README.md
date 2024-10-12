<h2 align=center> <b>krowbar</b> </h2>

<p align="center"> <img alt="GitHub release (latest SemVer)" src="https://github.com/user-attachments/assets/4257a032-eb0d-4dd7-a414-779e410d2c19"> </p>
<p align=center> Status bar made for BSPWM, focused on ease of use, stability and speed. </p>
<p align=center> <img alt="GitHub release (latest SemVer)" src="https://img.shields.io/github/v/release/bloznelis/krowbar"> <img alt="GitHub Workflow Status" src="https://img.shields.io/github/actions/workflow/status/bloznelis/krowbar/ci.yaml"> </p>

### Motivation
Generic status bars provide superior customization for the price of complexity, but I've always wanted a bar that just works out-of-the-box.

### Features
* Listens to BSPWM events directly via Unix socket
* Shows focused desktop window count, no more getting lost in monocle mode
* Urgent desktop support
* All widgets are written in Rust - forget slow scripts
* First class multi-monitor support

### Setup
Add this to your `bspwmrc`:
```
# Restarts krowbar, when restarting BSPWM. Allows for quick iteration, if configuring.
killall krowbar

# Regular BSPWM monitor setup, krowbar will use these as dekstop names
bspc monitor {your-monitor-name} -d web code III IV V VI

# Start krowbar
krowbar &
```

### krowbar might be for you if you:
- skipped on BSPWM, because it has no default status bar
- are drowning in semi-working configuration
- need a decently looking, working status bar, while searching for a nice [eww](https://github.com/elkowar/eww) config in [/r/unixporn](https://www.reddit.com/r/unixporn/)
- always wanted something akin to [i3status](https://i3wm.org/docs/i3status.html) but for BSPWM

### Showcase
![left](https://github.com/user-attachments/assets/29cbcf44-b4cf-4f09-b618-0b725ed2ddb1)
![right](https://github.com/user-attachments/assets/3c7e8e1b-cf36-4db0-b8f1-ce190e416115)

### TODO
- Prepare arch release
