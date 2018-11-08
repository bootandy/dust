
[![Build Status](https://travis-ci.org/bootandy/dust.svg?branch=master)](https://travis-ci.org/bootandy/dust)

# Dust

du + rust = dust. Like du but more intuitive

## Install

#### Cargo Install

* cargo install du-dust

#### Download Install

* Download linux / mac binary from [Releases](https://github.com/bootandy/dust/releases)
* unzip file: tar -xvf _downloaded_file.tar.gz_
* move file to executable path: sudo mv dust /usr/local/bin/

## Overview

Dust is meant to give you an instant overview of which directories are using disk space without requiring sort or head. Dust will print a maximum of 1 'Did not have permissions message'.

Dust will list the 20 biggest sub directories or files and will smartly recurse down the tree to find the larger ones. There is no need for a '-d' flag or a '-h' flag. The largest sub directory will have its size shown in *red*

## Why?

du has a number of ways of showing you what it finds, in terms of disk consumption, but really, there are only one or two ways you invoke it: with -h for “human readable” units, like 100G or 89k, or with -b for “bytes”. The former is generally used for a quick survey of a directory with a small number of things in it, and the latter for when you have a bunch and need to sort the output numerically, and you’re obligated to either further pass it into something like awk to turn bytes into the appropriate human-friendly unit like mega or gigabytes, or pipe thru sort and head while remembering the '-h' flag. Then once you have the top offenders, you recurse down into the largest one and repeat the process until you’ve found your cruft or gems and can move on.

Dust assumes that’s what you wanted to do in the first place, and takes care of tracking the largest offenders in terms of actual size, and showing them to you with human-friendly units and in-context within the filetree.

## Usage

```
Usage: dust <dir>
Usage: dust <dir>  <another_dir> <and_more>
Usage: dust -p <dir>  (full-path - does not shorten the path of the subdirectories)
Usage: dust -s <dir>  (apparent-size - shows the length of the file as opposed to the amount of disk space it uses)
Usage: dust -n 30  <dir>  (Shows 30 directories not 20)
Usage: dust -d 3  <dir>  (Shows 3 levels of subdirectories)
```

```
djin:git/dust> dust
 1.2G  target
 622M ├─┬ debug
 445M │ ├── deps
  70M │ ├── incremental
  56M │ └── build
 262M ├─┬ rls
 262M │ └─┬ debug
 203M │   ├── deps
  56M │   └── build
 165M ├─┬ package
 165M │ └─┬ du-dust-0.2.4
 165M │   └─┬ target
 165M │     └─┬ debug
 131M │       └── deps
 165M └─┬ release
 124M   └── deps
```

## Performance

Dust is currently about 4 times faster than du.

## Alternatives

* [NCDU](https://dev.yorhel.nl/ncdu)
* du -d 1 -h | sort -h

Note: Apparent-size is calculated slightly differently in dust to gdu. In dust each hard link is counted as using file_length space. In gdu only the first entry is counted.
