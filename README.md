# sengaka

線画化 - CLI tool to make images a line drawing, being suitable to anime captures or illustrations


![Example](example.png)

```
sengaka 0.1.0
CLI tool to make images a line drawing

USAGE:
    sengaka [OPTIONS] --if <FORMAT> --of <FORMAT>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -i, --input <FILE>       Input file path. If omit, standard input is used.
    -I, --if <FORMAT>        Input image format. e.g. png, jpg, ...
    -o, --output <FILE>      Output file path. If omit, standard output is used.
    -O, --of <FORMAT>        Output image format. e.g. png, jpg, ...
    -S, --shadow <shadow>    Shadow input level (0 ~ 255) [default: 150]
    -s, --sigma <sigma>      Blur sigma [default: 1.8]
```

## Install

**Windows**: Download from [releases](https://github.com/rot1024/sengaka/releases).

**MacOS or Linux**:

```sh
curl -L https://github.com/rot1024/sengaka/releases/download/v0.1.0/sengaka_0.1.0_`uname -s`_`uname -m` > /usr/local/bin/sengaka && chmod +x /usr/local/bin/sengaka
```

## Usage

```
sengaka -i foo.png -o bar.jpg
sengaka -i foo.png -O jpg > bar.jpg
sengaka -I png -o bar.jpg < foo.png
sengaka -I png -O jpg < foo.png > bar.jpg
```
