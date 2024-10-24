# Robin

A cross-platform command line tool to mass download manga and webnovel from multiple websites.

# Installation

## From Releases (Recommended)
Download the binary from [releases](https://github.com/NandeMD/robin) and use it.

## From Cargo
```bash
cargo install robin_cli
```

## From Source
You can get the latest features by compiling yourself.

1. Clone the repository.
    ```bash
    git clone https://github.com/NandeMD/robin.git
    ```
2. Change into directory.
    ```bash
    cd robin
    ```
3. Build and install.
    - With Cargo (recommended)
        ```bash
        cargo install
        ```
    - Custom
        ```bash
        cargo build

        # Afther the build process, you can locate robin(.exe) binary in the target/release folder.
        ```


# Usage
### Example:
```bash
# Simple bulk download:
robin -o ~/Desktop manga https://testurluwuowo.uwu

# Download 10 chapters at the same time (not recommended)
robin -o ~/Desktop -c 10 manga https://testurluwuowo.uwu

# Compress your download
robin -o ~/Desktop manga https://testurluwuowo.uwu --compress
```


# How to add site support?
- If you want a site added, please open an issue from [issue tracker](https://github.com/NandeMD/robin/issues).

- If you want to solve an issue or contribute the code, see [CONTRIBUTING.md](https://github.com/NandeMD/robin/blob/main/CONTRIBUTING.md)


# Supported Sites:
See [SITES.md](https://github.com/NandeMD/robin/blob/main/SITES.md).

# Goals
- [X] Add an option to manga command to select a chapter while downloading.
- [X] Add support (new command) for downloading webnovels.
- [X] Add a command option to webnovel command that converts the downloaded files to ePub format.
- [X] Add proxy option.
- [ ] Add at least 10 English content websites for manga.
- [ ] Add at least 10 English content websites for webnovel.
