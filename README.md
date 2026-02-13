# SwiftCore
SwiftCoreはRustで書かれた x86_64 / UEFI 向けのOSです。中学生によって開発/維持されています。

## ビルド
必要なツール:
    - git
    - homebrew
    - qemu-system-x86_64
    - x86_64-elf-gcc
    - cargo
    - rust nightly toolchain

1. このレポをクローンします。
2. サブモジュールをインストールします。
    ```bash
    git submodule update --init --recursive
    ```
3. libcのconfigureをします。
    ```bash
    cd src/lib
    ./configure
    ```
4. ビルドします。
    ```bash
    cargo build --release
    ```

[LICENSE](./LICENSE)ファイルを参照してください