
[![Build Status](https://travis-ci.org/bootandy/dust.svg?branch=master)](https://travis-ci.org/bootandy/dust)

# Dust

du + rust = dust. Like du but more intuitive

# Why

Because I want an easy way to see where my disk is being used.

# Demo
![Example](media/snap.png)

## Install

#### Cargo Install

* cargo install du-dust

#### Download Install

* Download linux / mac binary from [Releases](https://github.com/bootandy/dust/releases)
* unzip file: tar -xvf _downloaded_file.tar.gz_
* move file to executable path: sudo mv dust /usr/local/bin/

## Overview

Dust is meant to give you an instant overview of which directories are using disk space without requiring sort or head. Dust will print a maximum of 1 'Did not have permissions message'.

Dust will list a slightly-less-than-the-terminal-height number of the biggest sub directories or files and will smartly recurse down the tree to find the larger ones. There is no need for a '-d' flag or a '-h' flag. The largest sub directory will have its size shown in *red*

## Usage

```
Usage: dust
Usage: dust <dir>
Usage: dust <dir>  <another_dir> <and_more>
Usage: dust -p <dir>  (full-path - does not shorten the path of the subdirectories)
Usage: dust -s <dir>  (apparent-size - shows the length of the file as opposed to the amount of disk space it uses)
Usage: dust -n 30  <dir>  (Shows 30 directories not the default)
Usage: dust -d 3  <dir>  (Shows 3 levels of subdirectories)
Usage: dust -r  <dir>  (Reverse order of output, with root at the lowest)
Usage: dust -x  <dir>  (Only show directories on same filesystem)
Usage: dust -X ignore  <dir>  (Ignore all files and directories with the name 'ignore')
Usage: dust -b <dir>  (Do not show percentages or draw the ASCII bars)
```


## Alternatives

* [NCDU](https://dev.yorhel.nl/ncdu)
* [dutree](https://github.com/nachoparker/dutree)
* du -d 1 -h | sort -h

Note: Apparent-size is calculated slightly differently in dust to gdu. In dust each hard link is counted as using file_length space. In gdu only the first entry is counted.
