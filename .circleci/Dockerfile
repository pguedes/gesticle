FROM cimg/rust:1.46

RUN sudo apt-get update -yqq && \
    sudo apt-get install -yqq libxdo-dev libinput-dev libudev-dev libssl-dev && \
    cargo install cargo-deb cargo-release
