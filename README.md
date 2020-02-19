
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

Dust will list the terminal height - 10 biggest sub directories or files and will smartly recurse down the tree to find the larger ones. There is no need for a '-d' flag or a '-h' flag. The largest sub directory will have its size shown in *red*

## Why?

du has a number of ways of showing you what it finds, in terms of disk consumption, but really, there are only one or two ways you invoke it: with -h for “human readable” units, like 100G or 89k, or with -b for “bytes”. The former is generally used for a quick survey of a directory with a small number of things in it, and the latter for when you have a bunch and need to sort the output numerically, and you’re obligated to either further pass it into something like awk to turn bytes into the appropriate human-friendly unit like mega or gigabytes, or pipe thru sort and head while remembering the '-h' flag. Then once you have the top offenders, you recurse down into the largest one and repeat the process until you’ve found your cruft or gems and can move on.

Dust assumes that’s what you wanted to do in the first place, and takes care of tracking the largest offenders in terms of actual size, and showing them to you with human-friendly units and in-context within the filetree.

## Usage

```
Usage: dust
Usage: dust <dir>
Usage: dust <dir>  <another_dir> <and_more>
Usage: dust -p <dir>  (full-path - does not shorten the path of the subdirectories)
Usage: dust -s <dir>  (apparent-size - shows the length of the file as opposed to the amount of disk space it uses)
Usage: dust -n 30  <dir>  (Shows 30 directories not 20)
Usage: dust -d 3  <dir>  (Shows 3 levels of subdirectories)
Usage: dust -r  <dir>  (Reverse order of output, with root at the lowest)
Usage: dust -x  <dir>  (Only show directories on same filesystem)
Usage: dust -X ignore  <dir>  (Ignore all files and directories with the name 'ignore')
Usage: dust -b <dir>  (Do not show percentages or draw the ASCII bars)
```

```
$ dust  target
  15M     ┌── build                                    │                       ░█ │   2%
  25M     ├── deps                                     │                       ░█ │   4%
  45M   ┌─┴ release                                    │                       ██ │   7%
  84M   │   ┌── build                                  │                ▒▒▒▒▒████ │  13%
 7.6M   │   │ ┌── libsynstructure-f7552412787ad339.rlib│                ▒▒▒▓▓▓▓▓█ │   1%
  16M   │   │ ├── libfailure_derive-e18365d3e6be2e2c.so│                ▒▒▒▓▓▓▓▓█ │   2%
  18M   │   │ ├── libsyn-9ad95b745845d5dd.rlib         │                ▒▒▒▓▓▓▓▓█ │   3%
  19M   │   │ ├── libsyn-d4a3458fcb1c592c.rlib         │                ▒▒▒▓▓▓▓▓█ │   3%
 135M   │   ├─┴ deps                                   │                ▒▒▒██████ │  20%
 228M   │ ┌─┴ debug                                    │                █████████ │  34%
 228M   ├─┴ rls                                        │                █████████ │  34%
  18M   │ ┌── dust                                     │          ░░░░░░░░░░░░░░█ │   3%
  22M   │ ├── dust-a0c31c4633c5fc8b                    │          ░░░░░░░░░░░░░░█ │   3%
 7.4M   │ │   ┌── s-fkrj3vfncf-19aj951-1fv3o6tzvr348   │          ░░░░░░░░░░░░░▒█ │   1%
 7.4M   │ │ ┌─┴ dust-1i3xquz5fns51                     │          ░░░░░░░░░░░░░▒█ │   1%
  40M   │ ├─┴ incremental                              │          ░░░░░░░░░░░░░██ │   6%
  41M   │ ├── build                                    │          ░░░░░░░░░░░░░██ │   6%
 7.6M   │ │ ┌── libsynstructure-f7552412787ad339.rlib  │          ░░░░▒▒▒▒▒▒▒▒▒▒█ │   1%
 8.2M   │ │ ├── libserde-ab4b407a415bc8fc.rmeta        │          ░░░░▒▒▒▒▒▒▒▒▒▒█ │   1%
 9.4M   │ │ ├── libserde-ab4b407a415bc8fc.rlib         │          ░░░░▒▒▒▒▒▒▒▒▒▒█ │   1%
  11M   │ │ ├── tests_symlinks-bf063461b7be6a99        │          ░░░░▒▒▒▒▒▒▒▒▒▒█ │   2%
  11M   │ │ ├── integration-08f999d253e3b70c           │          ░░░░▒▒▒▒▒▒▒▒▒▒█ │   2%
  15M   │ │ ├── dust-1c6e63725d641738                  │          ░░░░▒▒▒▒▒▒▒▒▒▒█ │   2%
  16M   │ │ ├── libfailure_derive-e18365d3e6be2e2c.so  │          ░░░░▒▒▒▒▒▒▒▒▒▒█ │   2%
  18M   │ │ ├── dust-3a419f62b84d73c1                  │          ░░░░▒▒▒▒▒▒▒▒▒▒█ │   3%
  18M   │ │ ├── dust-2bdf724d4a721d31                  │          ░░░░▒▒▒▒▒▒▒▒▒▒█ │   3%
  18M   │ │ ├── libsyn-9ad95b745845d5dd.rlib           │          ░░░░▒▒▒▒▒▒▒▒▒▒█ │   3%
  23M   │ │ ├── libclap-0dedc35af3ef0670.rlib          │          ░░░░▒▒▒▒▒▒▒▒▒▒█ │   3%
 267M   │ ├─┴ deps                                     │          ░░░░███████████ │  40%
 392M   ├─┴ debug                                      │          ███████████████ │  59%
 667M ┌─┴ target                                       │█████████████████████████ │ 100%

```


## Alternatives

* [NCDU](https://dev.yorhel.nl/ncdu)
* [dutree](https://github.com/nachoparker/dutree)
* du -d 1 -h | sort -h

Note: Apparent-size is calculated slightly differently in dust to gdu. In dust each hard link is counted as using file_length space. In gdu only the first entry is counted.
