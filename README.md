[![Build Status](https://github.com/bootandy/dust/actions/workflows/CICD.yml/badge.svg)](https://github.com/bootandy/dust/actions)


# Dust

du + rust = dust. Like du but more intuitive.

# Why

Because I want an easy way to see where my disk is being used.

# Demo

<p align="center">
  <img src="media/demo.gif" alt="dust demo — interactive disk usage analysis" width="600">
</p>

![Example](media/snap.png)

Study the above picture. 

* We see `target` has 1.8G
* `target/debug` is the same size as `target` - so we know nearly all the disk usage of the 1.8G is in this folder
* `target/debug/deps` this is 1.2G - Note the bar jumps down to 70% to indicate that most disk usage is here but not all.
* `target/debug/deps/dust-e78c9f87a17f24f3` - This is the largest file in this folder, but it is only 46M - Note the bar jumps down to 3% to indicate the file is small.

From here we can conclude:
 * `target/debug/deps` takes the majority of the space in `target` and that `target/debug/deps` has a large number of relatively small files.
  

## Install

### Quick Install (Linux, macOS, Windows) 
```bash
curl -sSfL https://raw.githubusercontent.com/bootandy/dust/refs/heads/master/install.sh | sh
```

### Cargo <a href="https://repology.org/project/du-dust/versions"><img src="https://repology.org/badge/vertical-allrepos/du-dust.svg" alt="Packaging status" align="right"></a>

#### Cargo

- `cargo install du-dust`

#### 🍺 Homebrew (Mac OS)

- `brew install dust`

#### 🍺 Homebrew (Linux)

- `brew install dust`

#### DNF (Fedora Linux)

- `sudo dnf install du-dust`

#### [Snap](https://ubuntu.com/core/services/guide/snaps-intro) Ubuntu and [supported systems](https://snapcraft.io/docs/installing-snapd)

- `snap install dust`

Note: `dust` installed through `snap` can only access files stored in the `/home` directory. See danie-dejager/dust-snap#2 for more information.

#### [mise](https://github.com/jdx/mise)

- `mise use -g dust`

#### [Pacstall](https://github.com/pacstall/pacstall) (Debian/Ubuntu)

- `pacstall -I dust-bin`

#### Anaconda (conda-forge)

- `conda install -c conda-forge dust`

#### [deb-get](https://github.com/wimpysworld/deb-get) (Debian/Ubuntu)

- `deb-get install du-dust`

#### [x-cmd](https://www.x-cmd.com/pkg/#VPContent)

- `x env use dust`

#### Windows:

- `scoop install dust`
- Windows GNU version - works
- Windows MSVC - requires: [VCRUNTIME140.dll](https://docs.microsoft.com/en-gb/cpp/windows/latest-supported-vc-redist?view=msvc-170)

#### Download

- Download Linux/Mac binary from [Releases](https://github.com/bootandy/dust/releases)
- unzip file: `tar -xvf _downloaded_file.tar.gz`
- move file to executable path: `sudo mv dust /usr/local/bin/`

## Overview

Dust is meant to give you an instant overview of which directories are using disk space without requiring sort or head. Dust will print a maximum of one 'Did not have permissions message'.

Dust will list a slightly-less-than-the-terminal-height number of the biggest subdirectories or files and will smartly recurse down the tree to find the larger ones. There is no need for a '-d' flag or a '-h' flag. The largest subdirectories will be colored.

The different colors on the bars: These represent the combined tree hierarchy & disk usage. The shades of grey are used to indicate which parent folder a subfolder belongs to. For instance, look at the above screenshot. `.steam` is a folder taking 44% of the space. From the `.steam` bar is a light grey line that goes up. All these folders are inside `.steam` so if you delete `.steam` all that stuff will be gone too.

If you are new to the tool I recommend to try tweaking the `-n` parameter. `dust -n 10`, `dust -n 50`.

## Usage

```
Usage: dust
Usage: dust <dir>
Usage: dust <dir>  <another_dir> <and_more>
Usage: dust -p (full-path - Show fullpath of the subdirectories)
Usage: dust -s (apparent-size - shows the length of the file as opposed to the amount of disk space it uses)
Usage: dust -n 30  (Shows 30 directories instead of the default [default is terminal height])
Usage: dust -d 3  (Shows 3 levels of subdirectories)
Usage: dust -D (Show only directories (eg dust -D))
Usage: dust -F (Show only files - finds your largest files)
Usage: dust -r (reverse order of output)
Usage: dust -o si/b/kb/kib/mb/mib/gb/gib (si - prints sizes in powers of 1000. Others print size in that format).
Usage: dust -X ignore  (ignore all files and directories with the name 'ignore')
Usage: dust -x (Only show directories on the same filesystem)
Usage: dust -b (Do not show percentages or draw ASCII bars)
Usage: dust -B (--bars-on-right - Percent bars moved to right side of screen)
Usage: dust -i (Do not show hidden files)
Usage: dust -c (No colors [monochrome])
Usage: dust -C (Force colors)
Usage: dust --dim (Dim the percent bars to reduce brightness on dark terminals)
Usage: dust -f (Count files instead of diskspace [Counts by inode, to include duplicate inodes use dust -f -s])
Usage: dust -t (Group by filetype)
Usage: dust -z 10M (min-size, Only include files larger than 10M)
Usage: dust -e regex (Only include files matching this regex (eg dust -e "\.png$" would match png files))
Usage: dust -v regex (Exclude files matching this regex (eg dust -v "\.png$" would ignore png files))
Usage: dust -L (dereference-links - Treat sym links as directories and go into them)
Usage: dust -P (Disable the progress indicator)
Usage: dust -R (For screen readers. Removes bars/symbols. Adds new column: depth level. (May want to use -p for full path too))
Usage: dust -S (Custom Stack size - Use if you see: 'fatal runtime error: stack overflow' (default allocation: low memory=1048576, high memory=1073741824)"),
Usage: dust --skip-total (No total row will be displayed)
Usage: dust -z 40000/30MB/20kib (Exclude output files/directories below size 40000 bytes / 30MB / 20KiB)
Usage: dust -j (Prints JSON representation of directories, try: dust -j  | jq)
Usage: dust --files0-from=FILE (Read NUL-terminated file paths from FILE; if FILE is '-', read from stdin)
Usage: dust --files-from=FILE (Read newline-terminated file paths from FILE; if FILE is '-', read from stdin)
Usage: dust --collapse=node-modules will keep the node-modules folder collapsed in display instead of recursively opening it
```

## Config file

Dust has a config file where the above options can be set.
Either: `$XDG_CONFIG_HOME/dust/config.toml` (falling back to
`~/.config/dust/config.toml`) or `~/.dust.toml`
```
$ cat ~/.config/dust/config.toml
reverse=true
```

## Alternatives

- [NCDU](https://dev.yorhel.nl/ncdu)
- [dutree](https://github.com/nachoparker/dutree)
- [dua](https://github.com/Byron/dua-cli/)
- [pdu](https://github.com/KSXGitHub/parallel-disk-usage)
- [dirstat-rs](https://github.com/scullionw/dirstat-rs)
- `du -d 1 -h | sort -h`

## Why to use Dust over the Alternatives

Dust simply Does The Right Thing when handling lots of small files & directories. Dust keeps the output simple by only showing large entries.

Tools like ncdu & baobab, give you a view of directory sizes but you have no idea where the largest files are. For example directory A could have a size larger than directory B, but in fact the largest file is in B and not A. Finding this out via these other tools is not trivial whereas Dust will show the large file clearly in the tree hierarchy 

Dust will not count hard links multiple times (unless you want to `-s`).

Typing `dust -n 90` will show you your 90 largest entries. `-n` is not quite like `head -n` or `tail -n`, dust is intelligent and chooses the largest entries
