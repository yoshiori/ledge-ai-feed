# Ledge.ai RSS Feed Generator

Ledge.aiの最新記事を自動収集してRSS 2.0フィードを生成するRustプログラムです。

## 特徴

- **自動記事収集**: Ledge.aiから最新記事を自動取得
- **完全コンテンツ**: 記事の全文をRSSフィードに含む
- **自動更新**: GitHub Actionsで1時間ごとに自動実行
- **高性能**: Rustによる高速・安全な実装

## RSS フィード URL

以下のURLでRSSフィードにアクセスできます：

```
https://yoshiori.github.io/ledge-ai-feed/rss.xml
```

## 技術仕様

- **言語**: Rust (Edition 2021)
- **開発手法**: T-wadaスタイルTDD (テスト駆動開発)
- **自動化**: GitHub Actions (cron: 毎時実行)
- **フォーマット**: RSS 2.0

## 主な依存関係

- `rss` - RSS生成
- `scraper` - HTML解析
- `reqwest` - HTTP客户端
- `chrono` - 日付処理
- `pulldown-cmark` - Markdown → HTML変換

## ローカル実行

```bash
# 依存関係をインストール
cargo build

# RSSフィード生成
cargo run

# 生成されたフィードを確認
cat rss.xml
```

## 開発

このプロジェクトはT-wadaスタイルのTDD (テスト駆動開発) で構築されました：

1. **Red**: 失敗するテストを書く
2. **Green**: テストを通すコードを書く  
3. **Refactor**: コードを改善する

```bash
# テスト実行
cargo test

# フォーマット
cargo fmt

# Lint
cargo clippy
```

## 自動更新について

GitHub Actionsにより以下のスケジュールで自動実行されます：

- **定期実行**: 毎時0分に実行 (`cron: '0 * * * *'`)
- **手動実行**: GitHub UIから任意のタイミングで実行可能
- **コード変更時**: `src/`配下の変更をプッシュ時に実行

## ライセンス

MIT License - 詳細は [LICENSE](LICENSE) ファイルを参照してください。

---

🤖 このRSSフィードは自動生成されています。最新の技術情報をお楽しみください！