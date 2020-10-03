# gesticle

i saw someone talking about Rust and i never liked the solutions for swiping on my linux box... so i wrote this thing in a week on Christmas holidays to learn Rust.

since then this is actually what i use for my gestures on linux (both desktop and laptop)... it's not perfect but it's good enough for me to not bother with fixing it more :)

sharing is caring so i'm posting this to github hoping it may help someone else.

# dependencies

we need some libs to use this so please:
    
    > sudo apt install libxdo-dev libinput-dev libudev-dev

# create .deb installer

first we need to install cargo-deb

    > cargo install cargo-deb

and then we can:

    > cargo deb

# From manpage README

gesticle(1) -- Configurable handlers for gestures based on libinput events
==========================================================================

## SYNOPSIS

`gesticle` [-d] [-c *path*]

## DESCRIPTION

the **gesticle** application will send configurable key codes - via xdo - to the *X server* as a response to detected gestures built from libinput events.

Supported gestures are:

  - Swipes   3 and 4 finger swipes in directions *up*, *down*, *left* and *right*
  - Pinches  pinches in directions *in* and *out*
  - Rotation rotations in direction *left* and *right*

## CONFIGURATION

**gesticle** will check the configuration file based on the detected gesture and application
window with current focus and if not specified will default to the non-focused wndow specific setting.

For example, if the active application window is *gedit*, and the detected gesture
is a 3 finger swipe, the preferred setting will be
**swipe.down.3.gedit** and if that is not configured will look for a **swipe.down.3**

## FILES

    /etc/gesticle/config.toml system-wide configuration file
    ~/.gesticle/config.toml   user-specific configuration file

## SEE ALSO
  `libinput` (4)
