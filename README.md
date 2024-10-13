<h2 align=center> <b>krowbar</b> </h2>

<p align="center"> <img alt="GitHub release (latest SemVer)" src="https://github.com/bloznelis/krowbar/blob/master/images/krowbarx2.png"> </p>
<div align="center">

> "Oh, and before I forget, I think you dropped this back in Black Mesa!" [[1]](https://half-life.fandom.com/wiki/Crowbar)

</div>
<p align=center> Status bar made for BSPWM, focused on ease of use, stability and speed. </p>
<p align=center> <img alt="GitHub release (latest SemVer)" src="https://img.shields.io/github/v/release/bloznelis/krowbar"> <img alt="GitHub Workflow Status" src="https://img.shields.io/github/actions/workflow/status/bloznelis/krowbar/ci.yaml"> </p>


### Motivation
Generic status bars, while being complex, provide great customization, but I've always wanted a BSPWM bar that just works out-of-the-box.

### Features
* Listens to BSPWM events directly via Unix socket, i.e. instant updates
* Focused desktop window count widget, no more getting lost in monocle mode
* Urgent desktop support
* All widgets are written in Rust – forget slow scripts
* First class multi-monitor support
* In-built desktop, node count, active node name, network, cpu, mem, storage, battery, clock widgets

### Install
#### Cargo
`cargo install krowbar`

### Setup
Add this to your `bspwmrc`:
```
# Kill krowbar, when restarting BSPWM. Allows for quick iteration, if configuring.
killall krowbar

# Regular BSPWM monitor setup, krowbar will use these as dekstop names
bspc monitor {your-monitor-name} -d web code III IV V VI

# Start krowbar
krowbar &
```

### Config
`krowbar` looks for a config at `XDG_HOME/.config/krowbar/config.toml` or path passed via `--config`.

All values are optional, so in the config define only those you want to change (see Examples section).
``` toml
# Default values

[theme]
fg = "#ebc17a"
fg_dim = "#8b7653"
fg_bright = "#f7f7f7"
bg = "#1c1c1c"
bg_dim = "#232323"
ok = "#909d63"
ok_dim = "#5e6547"
alert = "#bc5653"
alert_dim = "#74423f"
warn = "#bc5653"
warn_dim = "#74423f"
bright = "#cacaca"
bright_dim = "#828282"
accent = "#bc5653"

[font]
font_family = "Terminess Nerd Font"
font_size = "16px"
font_weight = "bold"

[bar]
height = 30
position = "Bottom" # Top or Bottom
```

### Examples
#### krowbar classic
![](https://github.com/bloznelis/krowbar/blob/master/images/krowbar-classic-1.png)
![](https://github.com/bloznelis/krowbar/blob/master/images/krowbar-classic-2.png)

#### krowbar mute
![](https://github.com/bloznelis/krowbar/blob/master/images/krowbar-gray-1.png)
![](https://github.com/bloznelis/krowbar/blob/master/images/krowbar-gray-2.png)

```toml
[theme]
fg = "#cacaca"
fg_dim = "#828282"
```


#### krowbar moss
![](https://github.com/bloznelis/krowbar/blob/master/images/krowbar-moss-1.png)
![](https://github.com/bloznelis/krowbar/blob/master/images/krowbar-moss-2.png)

```toml
[theme]
fg = "#909d63"
fg_dim = "#5e6547"
accent = "#ebc17a"

[font]
font_family = "Terminess Nerd Font"
font_size = "12px"
font_weight = "bold"

[bar]
height = 20
```

### You might want to try out krowbar if you:
- Skipped on BSPWM, because it has no default status bar
- Are drowning in semi-working configuration
- Need a decently looking, functional status bar while searching for a nice [eww](https://github.com/elkowar/eww) config in [/r/unixporn](https://www.reddit.com/r/unixporn/)
- Always wanted something akin to [i3status](https://i3wm.org/docs/i3status.html) but for BSPWM

### TODO
- Prepare arch release
