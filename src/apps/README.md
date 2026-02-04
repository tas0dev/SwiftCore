# SwiftCore アプリケーション

このディレクトリには、SwiftCoreカーネル上で実行されるユーザーランドアプリケーションとドライバが含まれています。

## ディレクトリ構造

```
src/apps/
├── test_app/          # テストアプリケーション
└── (今後追加されるアプリ)
```

## アプリケーションの追加方法

1. **新しいCargoプロジェクトを作成**:
   ```bash
   cd src/apps
   cargo new --bin my_app
   cd my_app
   ```

2. **カスタムターゲットファイルをコピー**:
   ```bash
   cp ../test_app/x86_64-swiftcore.json .
   cp ../test_app/linker.ld .
   ```

3. **Cargo.tomlを設定**:
   ```toml
   [package]
   name = "my_app"
   version = "0.1.0"
   edition = "2021"

   [dependencies]
   # 依存関係を追加

   [profile.dev]
   panic = "abort"

   [profile.release]
   panic = "abort"
   opt-level = "z"
   lto = true
   ```

4. **main.rsを実装**:
   ```rust
   #![no_std]
   #![no_main]

   use core::panic::PanicInfo;

   #[no_mangle]
   pub extern "C" fn _start() -> ! {
       // アプリのコードをここに書く
       loop {}
   }

   #[panic_handler]
   fn panic(_info: &PanicInfo) -> ! {
       loop {}
   }
   ```

5. **カーネルをビルド**:
   ```bash
   cd ../../..  # SwiftCoreルートディレクトリに戻る
   cargo build --release
   ```

ビルドシステム（`build.rs`）が自動的に：
- `src/apps/`以下の全Cargoプロジェクトを検出
- 各アプリをリリースモードでビルド
- ビルドされたバイナリを`initfs/`にコピー
- `initfs.ext2`イメージに含める

## 利用可能なシステムコール

現在利用可能なシステムコールは`src/user/`以下に定義されています：

- `write(fd, buf)` - ファイルディスクリプタに書き込み
- `read(fd, buf)` - ファイルディスクリプタから読み込み（未実装）
- `exit(code)` - プロセスを終了
- `yield_now()` - CPUを他のタスクに譲る
- `get_ticks()` - タイマーティック数を取得
- `ipc_send(dest, value)` - IPCメッセージ送信
- `ipc_recv(sender_ptr)` - IPCメッセージ受信

## 注意事項

- 各アプリは`#![no_std]`環境で動作します
- 標準ライブラリは使用できません
- カスタムターゲット（`x86_64-swiftcore.json`）が必要です
- エントリーポイントは`_start`関数です
