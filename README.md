# Robin

A cross-platform command line tool to mass download manga and webnovel from multiple websites.

# Installation

## From Cargo (Recommended)
```bash
cargo install robin_cli
```

## From Releases
Download the binary from [releases](https://github.com/NandeMD/robin) and use it.

## From Source
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