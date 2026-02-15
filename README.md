# SwiftCore
SwiftCoreはRustで書かれた x86_64 / UEFI 向けのOSです。中学生によって開発/維持されています。

## ビルド
必要なツール:
    - git
    - qemu-system-x86_64
    - x86_64-elf-gcc
    - cargo
    - rustup
    - make
    - e2fsprogs
    - texinfo
    - build-essentialで入るすべてのツール
    - `x86_64-unknown-none`ターゲット
    - `x86_64-unknown-uefi`ターゲット

> [!TIP]
> x86_64-elf-gccは[homebrew](https://brew.sh/)でインストールすることを推奨します。また、brewをインストール時、`Run there commands in your terminal to add Homebrew to your PATH`と表示されたら、必ず指示に従ってください。

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