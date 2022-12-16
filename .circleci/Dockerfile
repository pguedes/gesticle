FROM cimg/rust:1.64

RUN sudo apt-get update -yqq && \
    sudo apt-get install -yqq libxdo-dev libinput-dev libudev-dev libssl-dev libcairo2-dev libdbus-1-dev libpango1.0-dev\
    libatk1.0-dev libgdk-pixbuf2.0-dev libgtk-3-dev && \
    cargo install cargo-deb cargo-release
