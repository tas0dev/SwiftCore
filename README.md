# SwiftCore
SwiftCore は Rust で書かれた x86_64 / UEFI 向けのハイブリッドカーネルです。
SwiftCoreOS に使用されています。（開発途中）

## 特徴
- **UEFI 起動**: 最新の UEFI ファームウェアで動作
- **64-bit x86 アーキテクチャ**: 完全な 64-bit サポート
- **ハイブリッドカーネル設計**: モノリシックとマイクロカーネルの利点を組み合わせ
- **メモリ安全性**: Rust の型システムによる安全性保証

## ビルド
```bash
cargo build
```

### 実行
```bash
cargo run
```

## ライセンス
[LICENSE](./LICENSE)ファイルを参照してください