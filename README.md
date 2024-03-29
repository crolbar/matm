<h1 align="center">
A tool to watch anime, movies, tv shows and read manga from the comfort of the terminal! 
</h1>

<h3 align="center">
    matm works by scraping 
    <a href="https://aniwatch.to">aniwatch.to</a>, 
    <a href="https://flixhq.to/">flixhq.to</a>,
    <a href="https://manganato.com/">manganato.com</a>
    then playing the video through 
    <a href="https://github.com/mpv-player/mpv">mpv</a> and 
    <a href="https://github.com/pwmt/zathura">zathura</a> for manga
</h3>

# Install
### You are at your own risk when using this tool!

## Linux

### Arch
Install from the AUR
```
yay -S matm-bin
```
if the release dosnt work there is a git version `matm-git`
- the histroy file is in `~/.local/state/matm`
- the cache dir for manga is `~/.cache/matm`

## Windows
no

## Manual

### Build
You can build it from source on any linux distribution. (make sure you have all dependencies)
```
git clone https://github.com/crolbar/matm
cd matm
cargo build --release
```



### Running
The binary is in `target/release` so you can:
```
cd target/release
./matm
```

If you get "permission denied" make sure the binary is executable
```
chmod +x matm
```

Then you can cp the binary into your $PATH
```
sudo cp matm /usr/bin
```
You can remove the cloned repo if you want
```
cd ../../..
rm -rf matm
```

# Dependencies
#### Usage
- `mpv`
- `zathura-cb`
- `fzf` (for versions before 2.0)
#### Make
- `git`
- `cargo`

### AUR packages:
```
yay -S --needed mpv zathura-cb git cargo
```

# Usage
<details><summary><b>Watching anime</summary>

```
matm ani
```
You can use `matm a` for short

#### Continue to watch from history
```
matm a -c
```

#### Watch the dubbed versioin
```
matm a --dub
```

#### Get the help menu
```
matm a --help
```

</details><details><summary><b>Watching movies or tv shows</summary>

```
matm mov
```
You can use `matm m` for short

#### Continue to watch from history
```
matm m -c
```

#### Use vlc insead of mpv (not recommended)
Sometimes takes a bit to load
```
matm m --vlc
```

#### Get the help menu
```
matm m --help
```
</details><details><summary><b>Reading manga</summary>

```
matm man
```
You can use `matm ma` for short

#### Continue to watch from history
```
matm ma -c
```

#### Clean the cache directory
```
matm ma --clean
```

#### Get the help menu
```
matm ma --help
```
</details><details><summary><b>Selector</summary>

- Exit: `Esc`, `Alt + q`, `ctrl + c`
- Up: `arrow-up`, `alt + k`, `shift + tab`, `scrollup`
- Down: `arrow-down`, `alt + j`, `tab`, `scrolldown``
- Top: `PageUp`, `Home`, `alt + g`
- Bottom: `PageDown`, `End`, `alt + shift + g`
- Select: `Enter`, `double left click`

</details>

# Uninstall
### AUR
```
yay -R matm-bin
```
or if you are using the git version `matm-git`


### Manual
you can basically remove the binary file
```
sudo rm $(which matm)
```
and state and cache folders
```
rm -rf ~/.local/state/matm
rm -rf ~/.cache/matm
```

# Credits
- [ani-cli](https://github.com/pystardust/ani-cli): inspiration for this project
- [justchokingaround](https://github.com/justchokingaround): mmm lobster
