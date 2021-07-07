
[![Build Status](https://travis-ci.org/bootandy/dust.svg?branch=master)](https://travis-ci.org/bootandy/dust)

# Dust

du + rust = dust. Like du but more intuitive.

# Why

Because I want an easy way to see where my disk is being used.

# Demo
![Example](media/snap.png)

## Install

#### Cargo <a href="https://repology.org/project/du-dust/versions"><img src="https://repology.org/badge/vertical-allrepos/du-dust.svg" alt="Packaging status" align="right"></a>

* `cargo install du-dust`

#### 🍺 Homebrew (Mac OS)

* `brew install dust`

#### 🍺 Homebrew (Linux)

* `brew tap tgotwig/linux-dust && brew install dust`

#### Download

* Download Linux/Mac binary from [Releases](https://github.com/bootandy/dust/releases)
* unzip file: `tar -xvf _downloaded_file.tar.gz`
* move file to executable path: `sudo mv dust /usr/local/bin/`

## Overview

Dust is meant to give you an instant overview of which directories are using disk space without requiring sort or head. Dust will print a maximum of one 'Did not have permissions message'.

Dust will list a slightly-less-than-the-terminal-height number of the biggest subdirectories or files and will smartly recurse down the tree to find the larger ones. There is no need for a '-d' flag or a '-h' flag. The largest subdirectories will be colored.

## Usage

```
Usage: dust
Usage: dust <dir>
Usage: dust <dir>  <another_dir> <and_more>
Usage: dust -p (full-path - Show fullpath of the subdirectories)
Usage: dust -s (apparent-size - shows the length of the file as opposed to the amount of disk space it uses)
Usage: dust -n 30 (shows 30 directories instead of the default [default is terminal height])
Usage: dust -d 3  (shows 3 levels of subdirectories)
Usage: dust -r  (reverse order of output)
Usage: dust -X ignore  (ignore all files and directories with the name 'ignore')
Usage: dust -b (do not show percentages or draw ASCII bars)
Usage: dust -i (do not show hidden files)
Usage: dust -c (No colors [monochrome])
Usage: dust -f (Count files instead of space)
```


## Alternatives

* [NCDU](https://dev.yorhel.nl/ncdu)
* [dutree](https://github.com/nachoparker/dutree)
* [dua](https://github.com/Byron/dua-cli/)
* [pdu](https://github.com/KSXGitHub/parallel-disk-usage)
* [dirstat-rs](https://github.com/scullionw/dirstat-rs)
* du -d 1 -h | sort -h

Note: Apparent-size is calculated slightly differently in dust to gdu. In dust each hard link is counted as using file_length space. In gdu only the first entry is counted.
