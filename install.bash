
get_latest_release() {
  curl --silent "https://api.github.com/repos/pguedes/gesticle/releases/latest" | # Get latest release from GitHub api
    grep '"tag_name":' |                                            # Get tag line
    sed -E 's/.*"([^"]+)".*/\1/'                                    # Pluck JSON value
}

version=$(get_latest_release)
number="${version:1}"

echo "installing https://github.com/pguedes/gesticle/releases/download/${version}/gesticled_${number}_amd64.deb"
wget "https://github.com/pguedes/gesticle/releases/download/${version}/gesticled_${number}_amd64.deb" -P /tmp

sudo apt install "/tmp/gesticled_${number}_amd64.deb"
