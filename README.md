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
There are no binary installations avaliable yet.

#### Arch
Build and install from the AUR:
```
yay -S matm-git
```

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


You can remove the cloned repo if you want

```
cd ..
rm -rf matm
```

### Running
The binary is in `target/release` so you can:
```
cd target/release
./matm
```

If you get "permission denied" make sure it binary is executable
```
chmod +x matm
```

Then you can cp the binary into your $PATH
```
sudo cp matm /usr/bin
```

# Dependencies
#### Usage
- `mpv`
- `zathura-cb`
- `fzf`
#### Make
- `git`
- `cargo`

```
yay -S --needed mpv zathura-cb fzf git cargo
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
</details>

# Uninstall
### Aur
```
yay -R matm-git
```

### Manual
you can just remove the binary file
```
sudo rm /usr/bin/matm
```

# Credits
- [ani-cli](https://github.com/pystardust/ani-cli): inspiration for this project
- [justchokingaround](https://github.com/justchokingaround): mmm lobster