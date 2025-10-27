# Test Fixtures

このディレクトリには、`count_lines` の統合テストやエンドツーエンドテストで使用するテストデータを配置します。

## 目的

- 実際のファイル構造をシミュレートしたテストケースの提供
- 再現可能なテストシナリオの実現
- エッジケースやコーナーケースのテストデータ管理

## 構造

```
fixtures/
├── sample_rust_project/     # Rustプロジェクトのサンプル
├── sample_mixed_project/    # 複数言語が混在するプロジェクト
├── edge_cases/              # エッジケース用のファイル
│   ├── empty_file.txt
│   ├── large_file.txt
│   └── binary_file.bin
└── config_samples/          # 設定ファイルのサンプル
```

## フィクスチャの追加方法

新しいテストフィクスチャを追加する際は、以下のガイドラインに従ってください：

1. **目的を明確にする**: フィクスチャ名から何をテストするのか分かるようにする
2. **最小限に保つ**: テストに必要な最小限のファイル構造にする
3. **ドキュメント化**: 複雑なフィクスチャには README.md を追加
4. **バイナリファイルは慎重に**: 必要な場合のみ、できるだけ小さいサイズで

## 使用例

```rust
use std::path::PathBuf;

#[test]
fn test_with_fixture() {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("sample_rust_project");
    
    // テストコード
}
```

## 注意事項

- フィクスチャファイルは Git で管理されます
- 大きなファイル（>100KB）は避けてください
- 実際のプロジェクトデータやセンシティブな情報を含めないでください