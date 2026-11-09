# クラウド開発環境セットアップ

この手順は`/workspace/konjac-Translation`のRust共通コアとPython契約QCを実行するためのもの。新しいGit worktreeは作らず、既存checkoutを使う。Tauri、OS SDK、アプリのサーバーは現時点で施工対象の実装がないため導入・起動しない。

## 再現用インストール

Cloud Linux x86_64ホストでRustupが未導入の場合、公式配布元のinstallerを固定SHA-256と照合する。RustupはRust配布署名も検証する。チェックサム、TLS、またはパッケージ検証を無効化しない。

```bash
#!/usr/bin/env bash
set -euo pipefail
cd /workspace/konjac-Translation

export RUSTUP_HOME=/workspace/.rustup
export CARGO_HOME=/workspace/.cargo
export PATH="$CARGO_HOME/bin:$PATH"

if ! command -v rustup >/dev/null 2>&1; then
  case "$(uname -s)-$(uname -m)" in
    Linux-x86_64) ;;
    *) echo "Pinned rustup bootstrap supports the managed Linux x86_64 host only." >&2; exit 1 ;;
  esac
  RUSTUP_BOOTSTRAP="$(mktemp)"
  trap 'rm -f "$RUSTUP_BOOTSTRAP"' EXIT
  curl --fail --silent --show-error --location --proto '=https' --tlsv1.2 \
    "https://static.rust-lang.org/rustup/dist/x86_64-unknown-linux-gnu/rustup-init" \
    --output "$RUSTUP_BOOTSTRAP"
  printf '%s  %s\n' \
    "b1c8cb14e30889e7e2999e6b69e62d9e9b9ffc0f4df8180564d306d9680e6c55" \
    "$RUSTUP_BOOTSTRAP" | sha256sum --check --status
  chmod 0700 "$RUSTUP_BOOTSTRAP"
  "$RUSTUP_BOOTSTRAP" -y --no-modify-path --profile minimal --default-toolchain 1.99.1
fi

rustup toolchain install 1.99.1 --profile minimal --component rustfmt --component clippy
rustup default 1.99.1
cargo fetch --locked

if [ ! -x /workspace/.venvs/konjac-translation/bin/python ]; then
  python3 -m venv /workspace/.venvs/konjac-translation
fi
/workspace/.venvs/konjac-translation/bin/python -m pip install --disable-pip-version-check -r requirements-dev.lock
/workspace/.venvs/konjac-translation/bin/python -m pip check
```

## 起動・検証

現時点で常駐プロセスはない。製品checkout直下で共通コア、境界QC、Python射影QCを実行する。

```bash
cd /workspace/konjac-Translation
export RUSTUP_HOME=/workspace/.rustup
export CARGO_HOME=/workspace/.cargo
export PATH="$CARGO_HOME/bin:$PATH"
cargo test --workspace --locked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
/workspace/.venvs/konjac-translation/bin/python -m unittest discover -s tests -v
/workspace/.venvs/konjac-translation/bin/python -m pip check
```

2026-11-09の現環境でRust 1.99.1テスト40件、Python 3.12テスト25件、Rust formatter、clippy、`pip check`がPASS。これはBCP 47構文・正規化、ConsentGrant/TranslationSession/participantのnullable項目・対象/remote同意・session revision/参照重複、共通論理型・ConsentGrant主体ID/ConsentRevocation実行者・理由コード境界・同意評価・grant ID重複拒否、SourceAssetのfingerprint/所有者・権利参照/取得同意/scalar metadata/参照長、SourceSpanのtagged payload/position/観測元、EngineExecutionの版付き能力証拠/input・response fingerprint/終端対応、TranslationRequestのnullable項目・整数幅・Unicodeキー長・参照重複、TranslationResultの進行・終端状態とnullable evidence/失敗コード境界、TranslationVariantの暫定/確定状態・実行証跡、DeliveryArtifactのpayload/宛先/配置参照と状態/同意/配送時刻条件、確定Variant・原文範囲・能力permit・成功EngineExecutionを結ぶ配送grant preflight、SourceAssetごとの取得/local・remote処理同意照合、session-scoped並行冪等claim、ページ間event cursorのsequence・終端/配送順、TranslationEvent nullable参照・u64 sequence上限・種別ごとのwire参照要件、セッションと入力版の参照境界、言語ペア/文脈/モード別能力境界、要求時間窓、拒否/期限/取消時の実行前停止、JSON/OpenAPI射影の局所確認であり、IANA registry有効性、エンジン対応、永続冪等台帳/event log、再起動復旧、実配信、認証サービス、永続同意、実行中取消伝播、MINIDORA実訳、アプリ、サービス、OS別QAや製品受入を示さない。

## クラウド設定の保存状態

現環境はLinux x86_64、接続中で、ネットワークポリシーは`package_managers`プリセットで強制適用されている。Rust配布/CargoとPython配布に必要な宛先は既存ポリシーで許可されており、この作業では追加ドメイン、認証情報、秘密値を要求していない。

オンボーディング設定ドラフトの読取と、`install_script`／`start_skill`の部分保存は、ツールから`INVALID_ARGUMENT`が返り永続化を確認できなかった。上記手順はcheckout内に保存した再利用資料であり、クラウド設定フィールドへの保存や次タスクへの自動適用を意味しない。将来タスクで自動復元するには、環境設定の`install_script`と`start_skill`に同じ手順を保存する必要がある。公開・環境snapshotの作成は未実施。
