> **语言**：本文档为简体中文翻译，[点此查看英文原版 README.md](README.md)。

[![Build Status](https://github.com/bootandy/dust/actions/workflows/CICD.yml/badge.svg?branch=master)](https://github.com/bootandy/dust/actions)


# Dust

du + rust = dust。类似 du，但更直观。

# 缘由

因为我想要一种简便的方式来查看磁盘空间都用在了哪里。

# 演示

<p>
  <img src="media/demo.gif" alt="dust demo — interactive disk usage analysis" width="600">
</p>


研究一下上面的演示

* 我们 cd 进入一个 webapp，它的大小为 72K。

* 我们看到 `src` 文件夹里有 32K。注意进度条跳到了 44%。webapp 有一半都在 src 里

* 我们看到 `src/utils` 里有 8K。进度条降到了 11%，但有一道阴影停留在 44%，表明它是 `src` 文件夹的一部分

* 然后我们演示 -d 2 参数 —— 显示 2 层文件夹的深度

* 我们演示 -n 5 参数 —— 只显示 5 个条目（默认情况下会显示终端高度个条目）


## 安装

### 快速安装（Linux、macOS、Windows）
```bash
curl -sSfL https://raw.githubusercontent.com/bootandy/dust/refs/heads/master/install.sh | sh
```

### Cargo <a href="https://repology.org/project/du-dust/versions"><img src="https://repology.org/badge/vertical-allrepos/du-dust.svg" alt="Packaging status" align="right"></a>

#### Cargo

- `cargo install du-dust`

#### 🍺 Homebrew（Mac OS）

- `brew install dust`

#### 🍺 Homebrew（Linux）

- `brew install dust`

#### DNF（Fedora Linux）

- `sudo dnf install du-dust`

#### [Snap](https://ubuntu.com/core/services/guide/snaps-intro) Ubuntu 及[受支持的系统](https://snapcraft.io/docs/installing-snapd)

- `snap install dust`

注意：通过 `snap` 安装的 `dust` 只能访问存放在 `/home` 目录中的文件。更多信息参见 danie-dejager/dust-snap#2。

#### [mise](https://github.com/jdx/mise)

- `mise use -g dust`

#### [Pacstall](https://github.com/pacstall/pacstall)（Debian/Ubuntu）

- `pacstall -I dust-bin`

#### Anaconda（conda-forge）

- `conda install -c conda-forge dust`

#### [deb-get](https://github.com/wimpysworld/deb-get)（Debian/Ubuntu）

- `deb-get install du-dust`

#### [x-cmd](https://www.x-cmd.com/pkg/#VPContent)

- `x env use dust`

#### Windows：

- `scoop install dust`
- Windows GNU 版本 —— 可用
- Windows MSVC —— 需要：[VCRUNTIME140.dll](https://docs.microsoft.com/en-gb/cpp/windows/latest-supported-vc-redist?view=msvc-170)

#### 下载

- 从 [Releases](https://github.com/bootandy/dust/releases) 下载 Linux/Mac 二进制文件
- 解压文件：`tar -xvf _downloaded_file.tar.gz`
- 将文件移动到可执行路径：`sudo mv dust /usr/local/bin/`

## 概览

Dust 旨在让你即刻了解哪些目录正在占用磁盘空间，而无需借助 sort 或 head。Dust 最多只会打印一条"权限不足"（Did not have permissions）提示。

Dust 会列出略少于终端高度数量的最大子目录或文件，并智能地沿目录树向下递归以找出较大的那些。无需使用 '-d' 参数或 '-h' 参数。最大的子目录会被着色。

进度条上不同的颜色：它们表示合并后的目录层级结构与磁盘占用。灰色深浅用于指示某个子文件夹属于哪个父文件夹。举例来说，看看上面的截图。`.steam` 是一个占据了 44% 空间的文件夹。从 `.steam` 的进度条延伸出一条浅灰色的线连到上方。所有这些文件夹都在 `.steam` 之内，所以如果你删除了 `.steam`，那些内容也都会一并消失。

如果你是初次接触本工具，我建议尝试调整 `-n` 参数。`dust -n 10`、`dust -n 50`。

## 用法

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
Usage: dust -T (Set the number of walker threads. Default is the CPU count. For high-latency storage like NFS or remote mounts, a higher count can speed up the walk by overlapping more concurrent stat calls.)
Usage: dust -R (For screen readers. Removes bars/symbols. Adds new column: depth level. (May want to use -p for full path too))
Usage: dust --skip-total (No total row will be displayed)
Usage: dust -z 40000/30MB/20kib (Exclude output files/directories below size 40000 bytes / 30MB / 20KiB)
Usage: dust -j (Prints JSON representation of directories, try: dust -j  | jq)
Usage: dust --files0-from=FILE (Read NUL-terminated file paths from FILE; if FILE is '-', read from stdin)
Usage: dust --files-from=FILE (Read newline-terminated file paths from FILE; if FILE is '-', read from stdin)
Usage: dust --collapse=node-modules will keep the node-modules folder collapsed in display instead of recursively opening it
Usage: dust --dim Displays dimmer bars

```

## 配置文件

Dust 有一个配置文件，可以在其中设置上述选项。
可以是：`$XDG_CONFIG_HOME/dust/config.toml`（回退到
`~/.config/dust/config.toml`）或 `~/.dust.toml`
```
$ cat ~/.config/dust/config.toml
reverse=true
limit-filesystem=true
```
键使用 kebab-case 风格的长参数名。更完整的示例参见 [config/config.toml](config/config.toml)。

## 替代品

- [NCDU](https://dev.yorhel.nl/ncdu)
- [dutree](https://github.com/nachoparker/dutree)
- [dua](https://github.com/Byron/dua-cli/)
- [pdu](https://github.com/KSXGitHub/parallel-disk-usage)
- [dirstat-rs](https://github.com/scullionw/dirstat-rs)
- `du -d 1 -h | sort -h`

## 为什么选择 Dust 而非上述替代品

在处理大量小文件与目录时，Dust 总能简单地做"正确的事"。Dust 通过只展示较大的条目，让输出保持简洁。

ncdu、baobab 这类工具会给你一种目录大小的视图，但你无从得知最大的文件在哪里。举例来说，目录 A 的大小可能大于目录 B，但实际上最大的文件在 B 而不在 A。用这些其他工具要查出这一点并不容易，而 Dust 会在目录层级结构中清晰地展示出那个大文件

Dust 不会重复计算硬链接（除非你想用 `-s`）。

输入 `dust -n 90` 会显示你最大的 90 个条目。`-n` 与 `head -n` 或 `tail -n` 并不完全相同，dust 很智能，它会挑选出最大的条目
