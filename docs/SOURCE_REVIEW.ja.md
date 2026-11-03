# 参照ソフトウェア解析・移植境界

- 観測根拠：GitHub公開コード・公式文書。確認日：2026-11-04。
- 目的：**こんにゃく翻訳の外側に必要なアーキテクチャ**を抽出すること。MINIDORA内部の代替エンジン探索ではない。
- 区別：**既存コードで観測した内容**と**Konjacへの採用提案**は別欄に分ける。採用は未実施。

## 1. 比較と採用候補

| 参照元 | 観測できる技術／挙動 | Konjac側で参考にする部分 | 検証を要する制約 |
|---|---|---|---|
| [RTranslator](https://github.com/niedev/RTranslator/tree/v3.00) | Android音声サービス、録音・VAD・ASR、翻訳器、システムTTS、トランシーバー型会話、Bluetooth端末間会話、モデルDL/設定/UI | スマホ音声/会話UI、音声ライフサイクル、Bluetoothの接続設計、TTS、権限、モデル資源管理 | Android特化。ASR/翻訳が密結合。MINIDORAへ置換可能な独立境界に整理が必要 |
| [Haas-ASR](https://github.com/third-intelligence/Haas-ASR) | 日本語音声認識モデルと長時間録音／疑似リアルタイムWebデモ。暫定字幕・確定字幕、音声区間検出、進捗・遅延表示 | 音声区間・時刻管理、WebSocket、暫定→確定イベント、音声入力UI | ASRであり翻訳LLMではない。デモは本番仕様ではなく、公式実行環境は主にNVIDIA GPU |
| [speech.cpp](https://github.com/nyosegawa/speech.cpp) | C/C++のモデルロード・認識・音声合成・検出API、HTTP音声エンドポイント・Realtime WebSocket、ステータス／キャンセル | Native speech adapter、局所ASR/TTS/VAD能力、サービスプロトコル、型付きエラー | モデルごとに実際の対応・性能・配布条件が異なる。音声専用で文書/翻訳体験全体は担わない |
| [MINIDORA-BETA](https://github.com/gatchimuchio/MINIDORA-BETA) | 入力機／駆動コア／出力機を備えるβ公開実装、限定受入テストと公開I/O | **唯一の選択済み駆動エンジン**との接続契約 | 任意言語ペア翻訳・実時間翻訳・クロス端末翻訳は未実証 |

なお「三者ともLLMで動く」は厳密には成立しない。RTranslatorの認識はWhisper系、翻訳は専用翻訳モデル群、HaasはASRモデルである。共通に抽出するべきは**音声／文書／通信／UIを扱う外部機構と、翻訳駆動との交換点**。

## 2. RTranslatorの具体的な観測箇所

基準：`v3.00` ブランチ、参照commit `ae96b008e31323ba8b75f94e8b5d4c29b8c1307e`（取得時点のスナップショット）。

| 公開コード | 抽出対象 |
|---|---|
| [VoiceTranslationService.java](https://github.com/niedev/RTranslator/blob/ae96b008e31323ba8b75f94e8b5d4c29b8c1307e/app/src/main/java/nie/translator/rtranslator/voice_translation/VoiceTranslationService.java) | フォアグラウンドサービス、入力/出力切替、UIへのイベント通知、音声ライフサイクル |
| [Recorder.java](https://github.com/niedev/RTranslator/blob/ae96b008e31323ba8b75f94e8b5d4c29b8c1307e/app/src/main/java/nie/translator/rtranslator/voice_translation/neural_networks/voice/Recorder.java) | 録音バッファ・VAD・発話境界とマイク停止/再開 |
| [Translator.java](https://github.com/niedev/RTranslator/blob/ae96b008e31323ba8b75f94e8b5d4c29b8c1307e/app/src/main/java/nie/translator/rtranslator/voice_translation/neural_networks/translation/Translator.java) | 部分訳／確定訳のコールバック、言語ペア・出力の情報。既存モデルとの結合は直接採用しない |
| [ConversationService.java](https://github.com/niedev/RTranslator/blob/ae96b008e31323ba8b75f94e8b5d4c29b8c1307e/app/src/main/java/nie/translator/rtranslator/voice_translation/_conversation_mode/_conversation/ConversationService.java) | Bluetooth端末間テキスト受け渡しと会話表示。インターネット通話とは区別 |
| [TTSEngine.java](https://github.com/niedev/RTranslator/blob/ae96b008e31323ba8b75f94e8b5d4c29b8c1307e/app/src/main/java/nie/translator/rtranslator/voice_translation/TTSEngine.java) | Android TTSの再生境界 |
| [LICENSE.txt](https://github.com/niedev/RTranslator/blob/v3.00/LICENSE.txt) / [NOTICE.txt](https://github.com/niedev/RTranslator/blob/v3.00/NOTICE.txt) | Apache-2.0の権利・帰属要件 |

RTranslatorのデータログ、モデル依存、Bluetooth通信、デバッグ署名などは本番流用に際して安全・機密・ビルド監査が必要。コードの丸ごと移植は本書の承認範囲外。

## 3. Haas-ASRの具体的な観測箇所

基準：`main`、参照commit `acc614f68769dcbfae9dba96b48439c4a66295af`。

- [demos/README.md](https://github.com/third-intelligence/Haas-ASR/blob/acc614f68769dcbfae9dba96b48439c4a66295af/demos/README.md)：公式に**非ストリーミングASRを約250msおきに再認識**する疑似ライブデモと説明。`0.7秒`無音または`14秒`上限で区間確定。**非製品向けデモ**と明記。
- [demos/realtime.py](https://github.com/third-intelligence/Haas-ASR/blob/acc614f68769dcbfae9dba96b48439c4a66295af/demos/realtime.py)：音声ストリーム、暫定/確定更新、測時。
- [demos/segmenter.py](https://github.com/third-intelligence/Haas-ASR/blob/acc614f68769dcbfae9dba96b48439c4a66295af/demos/segmenter.py)：音声区間分割。固定音量閾値の雑音・小声への弱点を確認する。
- [demos/longform.py](https://github.com/third-intelligence/Haas-ASR/blob/acc614f68769dcbfae9dba96b48439c4a66295af/demos/longform.py)：長尺音声の分割と時刻付き進捗通知。

Haas-ASRで観測された日本語認識率やGPU速度を、こんにゃく翻訳の全言語翻訳品質やAndroid速度として転用しない。

## 4. speech.cpp の具体的な観測箇所

基準：`main`、参照commit `03555100287c118b41d1b475044e5ab1e07baa8c`。

- [docs/c-api.md](https://github.com/nyosegawa/speech.cpp/blob/03555100287c118b41d1b475044e5ab1e07baa8c/docs/c-api.md)：モデル・要求の生成/破棄、`speech_request_set_audio`、`speech_transcribe`、`speech_synthesize`、型・タイムスタンプ・中断・エラー。
- [docs/server.md](https://github.com/nyosegawa/speech.cpp/blob/03555100287c118b41d1b475044e5ab1e07baa8c/docs/server.md)：`/v1/audio/transcriptions`、`/v1/audio/speech`、`/v1/realtime`などの接続形態。

本書はインターフェースの参考とするだけで、C++/ggml・音声モデルをKonjacへ組み込んでいない。

## 5. ライセンス・導入ゲート

- RTranslator、Haas-ASR：Apache-2.0。speech.cpp：MIT。**第三者モデルの重み、音声データ、UI素材、依存ライブラリには別条件があり得る**。
- 現時点で、これらの第三者のソース・モデル・データを本リポへコピーしていない。`NOTICE`には参照元として記録する。
- 取り込み前に、正確な版／出典／ファイル名／変更箇所／ライセンス条項／元NOTICE／再配布義務／動作環境を固定し、出典と採用判断を更新する。
- 参照元の特徴を**取り込めること**と、Konjacに**取り込まれ実装されたこと**を別に記録する。

## 6. 判定

**採用するのは、翻訳体験に必要な外部構成の設計パターン。** MINIDORAを据える意味と直接競合する既存翻訳エンジンの無断内蔵、他製品の過大な移植、音声機能限定のアーキテクチャ化は行わない。

