# bitshave

**小さくできても、デコード結果が変わる画像は採用しない。**

bitshave は外部ツールで PNG / JPEG / WebP / GIF を最適化する Rust CLI です。元より小さく、形式が同じで、デコード後の画素が一致した場合だけ出力します。GIF では全フレーム、各フレームの表示時間、ループ回数も比較します。改善がなければ元ファイルを残します。

## 保証すること・しないこと

- PNG / JPEG / 静止 WebP: このプログラムのデコーダーで得た RGBA 画素を比較します。GIF: 全フレームの画素・表示時間・ループ回数を比較します。デコード不能な候補は採用しません。
- 0 バイト、形式違い、サイズが同じか増えた候補は採用しません。出力先に一時ファイルを作ってから置き換え、書き込み失敗は成功扱いしません。
- アニメーション PNG / WebP はスキップします。SVG は描画結果の同等性を検証できないため、v0.1.1 では認識してもスキップします。
- メタデータの保持、別のデコーダーでの表示一致、色プロファイルや EXIF 回転を含む見た目の一致は保証しません。メタデータが必要な画像は事前にバックアップし、`--dry-run` で確認してください。
- WebP は `cwebp -lossless` によりデコード後の画素から再エンコードされます。元の WebP の圧縮方式やバイナリ構造を保持する意味ではありません。

## 開発・実行

Nix 開発シェルには `oxipng`、`jpegtran`、`cwebp`、`gifsicle` が含まれます。通常シェルでも実行できますが、対象形式の外部ツールが PATH に必要です。足りないツールがある形式はスキップされます。

```bash
nix develop --command cargo test --all-targets
nix develop --command cargo clippy --all-targets -- -D warnings
nix develop --command cargo run --release -- --help
```

```bash
# 現在のディレクトリの画像をその場で最適化
bitshave

# 指定ディレクトリを再帰処理し、改善した結果を別フォルダへ
bitshave --input ./assets --output ./dist --recursive

# 変更せずに結果を確認
bitshave --input ./assets --dry-run
```

`--output` 指定時は、改善したファイルだけ出力します。`--dry-run` でも実際に外部ツールを実行して候補を検証します。元ファイルを直接変更する用途では、通常のバックアップを推奨します。

## ライセンス

MIT License
