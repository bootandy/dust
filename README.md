# Dust
du + rust = dust. A rust alternative to du

Unlike du, dust is meant to give you an instant overview of which directories are using disk space without requiring sort or head. Dust does not count file system blocks; it uses file sizes instead. Dust will print a maximum of 1 'Did not have permissions message'.


Dust will list the 15 biggest sub directories and will smartly recurse down the tree to find the larger ones. There is no need for a '-d' flag or a '-h' flag. The largest sub directory will have its size shown in red

```
Usage: dust <dir>
Usage: dust -n 30  <dir>  (Shows 30 directories not 15)
```


```
dust .
 161M  .
 160M └── ./target
 123M    ├── ./target/debug
  83M    │  ├── ./target/debug/deps
  16M    │  │  ├── ./target/debug/deps/libclap-82e6176feef5d4b7.rlib
 8.6M    │  │  └── ./target/debug/deps/dust-993f7d919d92f0f8.dSYM
 8.6M    │  │     └── ./target/debug/deps/dust-993f7d919d92f0f8.dSYM/Contents
 8.6M    │  │        └── ./target/debug/deps/dust-993f7d919d92f0f8.dSYM/Contents/Resources
  27M    │  ├── ./target/debug/incremental
  12M    │  └── ./target/debug/build
  20M    ├── ./target/x86_64-apple-darwin
  20M    │  └── ./target/x86_64-apple-darwin/debug
  20M    │     └── ./target/x86_64-apple-darwin/debug/deps
  16M    │        └── ./target/x86_64-apple-darwin/debug/deps/libclap-7e3f8513c52cd558.rlib
  16M    └── ./target/release
  13M       └── ./target/release/deps
```

Performance: dust is currently about 4 times slower than du.




