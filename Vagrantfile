# -*- mode: ruby -*-
# vi: set ft=ruby :

Vagrant.configure(2) do |config|
  config.vm.box = "GR360RY/trusty64-desktop-minimal"

  config.vm.provider "virtualbox" do |vb|
    # Display the VirtualBox GUI when booting the machine
    vb.gui = true
  end

  config.vm.provision "shell", inline: <<-SHELL
    sudo add-apt-repository -y ppa:bugs-launchpad-net-falkensweb/cool-retro-term
    sudo add-apt-repository -y ppa:hansjorg/rust
    sudo apt-get update
    sudo apt-get install -y libncurses{,w}5-dev cool-retro-term cargo-nightly rust-nightly git
  SHELL

  # Build as unprivileged user
  config.vm.provision "shell", privileged: false, inline: <<-SHELL
    cd
    git clone https://github.com/euclio/robco-term
    cd robco-term
    cargo build --release
    echo "setxkbmap us" >> ~/.bashrc
    cat << EOF > ~/Desktop/robco-term.desktop
#!/usr/bin/env xdg-open

[Desktop Entry]
Version=0.0.1
Type=Application
Terminal=false
Exec=/usr/bin/cool-retro-term --workdir robco-term -e cargo run --release
Name=Robco Terminal
Icon=gnome-terminal
Path=/home/vagrant
EOF
    chmod +x ~/Desktop/robco-term.desktop
  SHELL
end
