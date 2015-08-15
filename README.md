# robco-term

A recreation of the [hacking mini-game] from Fallout 3 and Fallout: New
Vegas written in Rust using ncurses.

![screenshot](http://i.imgur.com/TgXwbnK.png)

## Installation

First, clone the repository.

```sh
$ git clone https://github.com/euclio/robco-term.git
```

OS X and Linux users have the choice between a native install and Vagrant (I
recommend native). Windows users must use Vagrant.

### Vagrant

If you don't want to bother installing dependencies, you can try running the
game inside a Vagrant environment. The environment also provides
[cool-retro-term].

First, install [Vagrant] and [VirtualBox]. Then, open a terminal or command
prompt inside the robco-term folder.

```sh
$ vagrant up
```

This will open a virtual machine and begin provisioning a simple Ubuntu
environment. The game will be ready to play when you see "Robco Terminal" on the
VM's desktop. Please note that this will take about 15 minutes and download
about 4GB on the first run. You can track progress in your terminal.

### Native

This program requires an installation of stable [Rust][rust-official] and
[cargo], and [ncurses].

If you can't get rust or cargo from your friendly neighborhood package manager,
try this script from the [official site][rust-install]:

```sh
$ curl -sSf https://static.rust-lang.org/rustup.sh | sh
```

Then, to install ncurses:

## OS X

First, install [homebrew]. Then, open a terminal.

```sh
$ brew install rust ncurses
```

## Linux (Ubuntu)

```sh
$ sudo apt-get update
$ sudo apt-get install libncurses{,w}5-dev
```

### Running the Code

```sh
$ cargo run --release
```

I recommend running the game in [cool-retro-term] to feel like you're
actually getting your Science skill to 100.

```sh
$ cool-retro-term -e cargo run --release
```

## Playing the game

A full description of how to play the hacking game can be found
[here][hacking mini-game].

### Controls

It's probably easiest to use the mouse to select the password you want to guess.
However, if you really want to feel like a hacker, you can also play the game
with <kbd>W</kbd> <kbd>A</kbd> <kbd>S</kbd> <kbd>D</kbd> and <kbd>Enter</kbd>.
If you're playing this on an [ADM-3A], <kbd>H</kbd> <kbd>J</kbd> <kbd>K</kbd>
<kbd>L</kbd> are also supported.

### Options

```
$ cargo run --release -- --help
Usage:
    robco-term [options]
    robco-term (-h | --help)

Options:
    -h --help                       Show this screen.
    -d LEVEL --difficulty=LEVEL     Set difficulty of the game (default 5). Currently this only
                                    affects the length of potential passwords.
```

## Limitations

* The game is pretty hard. Potential passwords are picked randomly from the
  system dictionary, meaning that most words have very few letters in common.
* In the same vein, there is no filter on the potential passwords, so you might
  get some less than tasteful words.
* This program is in no way meant to be an exact replica of the game. That said,
  I'd love any pull requests that improve the authenticity.
* As of this writing, cool-retro-term does not support xterm-1003 mouse mode
  ([cool-retro-term#251]). robco-term will still work, but highlighting on
  hover will not.

## TODO

* Add animations
* Add a graphical frontend (?)

## Legal

I'm not affiliated with Bethesda, Zenimax, or Obsidian in any way, just to be
clear. I made this as a labor of love, not to make any money!

[VirtualBox]: https://www.virtualbox.org/
[Vagrant]: https://www.vagrantup.com/
[homebrew]: http://brew.sh/
[rust-official]: https://www.rust-lang.org/
[rust-install]: https://www.rust-lang.org/install.html
[cargo]: https://crates.io
[ncurses]: http://www.gnu.org/software/ncurses/
[hacking mini-game]: http://fallout.wikia.com/wiki/Hacking#Hacking_terminals
[cool-retro-term]: https://github.com/Swordfish90/cool-retro-term
[cool-retro-term#251]: https://github.com/Swordfish90/cool-retro-term/issues/251
[ADM-3A]: https://en.wikipedia.org/wiki/ADM-3A
