# gesticle

i saw someone talking about Rust and i never liked the solutions for swiping on my linux box... so i wrote this thing in a week on Christmas holidays to learn Rust.

since then this is actually what i use for my gestures on linux (both desktop and laptop)... it's not perfect but it's good enough for me to not bother with fixing it more :)

sharing is caring so i'm posting this to github hoping it may help someone else.

## gesticle-gui

there's a gui now... because i wanted to play with gtk-rs... 

![oops no screenshot for you](https://github.com/pguedes/gesticle/blob/gui-tests/deb-assets/gesticle-gui-screenshot.png?raw=true "gesticle-gui")

it should make it easy to edit the gesture action configurations

## building from source

### clone this repo

    git clone https://github.com/pguedes/gesticle.git

### install dependencies

we need some libs to use this so please:
    
    sudo apt install libxdo-dev libinput-dev libudev-dev libssl-dev libcairo2-dev libdbus-1-dev libpango1.0-dev libatk1.0-dev libgdk-pixbuf2.0-dev libgtk-3-dev

### build with cargo

    cd gesticle
    cargo build

## create .deb installer

first we need to install cargo-deb

    cargo install cargo-deb

and then we can:

    cargo deb

