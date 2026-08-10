# 第II部 conformance — `meta` とは何か

## 説明
`meta` は、仕様見出しのうち**いまの実装（Rust クレート）に直接対応するコード表面がない**ものを追跡するためのステータスである。散文の用語定義、適用範囲の宣言、OPEN への移管表、Progress/Preservation などのメタ理論、完成判定の文章などが典型。「実装が足りない」(`gap`) とも「一部足りない」(`partial`) とも違い、**適合義務そのものがコードに落ちない**（または N/A）ことを示す。チェックリストから落とさず、仕様の骨格として残すために印す。

### 件数
- **meta 合計**: 132

---

## 一覧

### L241: 第II部 RPX言語の基礎仕様
- **なぜ meta**: 用語定義
- **notes**: 用語・カタログのみ

### L243: 共通用語と記号
- **なぜ meta**: 用語定義
- **notes**: 用語・カタログのみ

### L263: 10.1 Effect用語
- **なぜ meta**: 用語定義
- **notes**: 用語・カタログのみ

### L272: 記号一覧
- **なぜ meta**: 用語定義
- **notes**: 用語・カタログのみ

### L294: 言語機能の一覧
- **なぜ meta**: 用語定義
- **notes**: 用語・カタログのみ

### L320: 12.1 依存関係の要約
- **なぜ meta**: 用語定義
- **notes**: 用語・カタログのみ

### L338: 言語機能の詳細仕様
- **なぜ meta**: 用語定義
- **notes**: 用語・カタログのみ

### L379: 静的・動的意味
- **なぜ meta**: メタ理論
- **notes**: CSTに意味論なし（該当なし）

### L402: Progress/Preservation・その他
- **なぜ meta**: メタ理論
- **notes**: 理論性質; Core対象外

### L2540: 21. 本項目で意図的に確定しない事項
- **なぜ meta**: プロセス
- **notes**: 意図的スコープ外リスト（DAT/MOD等）

### L2569: 22. 状態
- **なぜ meta**: 完成判定の文章
- **notes**: OPEN-SYN-002 RESOLVED 状態表

### L2648: DD-001 決定概要
- **なぜ meta**: プロセス
- **notes**: scope/overview prose

### L2650: DD-001.1 状態
- **なぜ meta**: プロセス
- **notes**: scope/overview prose

### L2668: DD-001.2 中心的な決定
- **なぜ meta**: プロセス
- **notes**: scope/overview prose

### L2685: DD-001.3 宣言名
- **なぜ meta**: プロセス
- **notes**: scope/overview prose

### L2712: 0. 適用範囲
- **なぜ meta**: プロセス
- **notes**: scope/overview prose

### L2714: 0.1 本項目が定めるもの
- **なぜ meta**: プロセス
- **notes**: scope/overview prose

### L2737: 0.2 本項目が定めないもの
- **なぜ meta**: プロセス
- **notes**: scope/overview prose

### L4291: 22. 移管先OPEN・下位項目・状態
- **なぜ meta**: プロセス
- **notes**: spec OPEN/status prose; no direct code surface

### L4293: 22.1 `OPEN-SYN-002`への追補
- **なぜ meta**: プロセス
- **notes**: spec OPEN/status prose; no direct code surface

### L4303: 22.2 `OPEN-MOD-001`
- **なぜ meta**: プロセス
- **notes**: spec OPEN/status prose; no direct code surface

### L4317: 22.3 `OPEN-SEM-001`
- **なぜ meta**: プロセス
- **notes**: spec OPEN/status prose; no direct code surface

### L4328: 22.4 `OPEN-DERIVE-001`
- **なぜ meta**: プロセス
- **notes**: spec OPEN/status prose; no direct code surface

### L4341: 22.5 `OPEN-GADT-001`
- **なぜ meta**: プロセス
- **notes**: spec OPEN/status prose; no direct code surface

### L4355: 22.6 `OPEN-DYNAMIC-001`
- **なぜ meta**: プロセス
- **notes**: spec OPEN/status prose; no direct code surface

### L4369: 22.7 `OPEN-LAZY-001`
- **なぜ meta**: プロセス
- **notes**: spec OPEN/status prose; no direct code surface

### L4381: 22.8 `OPEN-GRAPH-001`
- **なぜ meta**: プロセス
- **notes**: spec OPEN/status prose; no direct code surface

### L4395: 22.9 下位項目
- **なぜ meta**: プロセス
- **notes**: spec OPEN/status prose; no direct code surface

### L4435: 22.10 最終状態
- **なぜ meta**: 完成判定の文章
- **notes**: spec OPEN/status prose; no direct code surface

### L4460: 状態
- **なぜ meta**: プロセス
- **notes**: EVAL-001 status prose (解決済み)

### L4839: 未決定事項の移管
- **なぜ meta**: プロセス
- **notes**: OPEN transfer table out of EVAL-001

### L5038: 状態
- **なぜ meta**: プロセス
- **notes**: BND status/principles prose

### L6669: 関連する後続設計課題
- **なぜ meta**: プロセス
- **notes**: OPEN / future-work prose

### L6671: `OPEN-SYN-002`
- **なぜ meta**: プロセス
- **notes**: OPEN / future-work prose

### L6682: `OPEN-TYP-002`
- **なぜ meta**: プロセス
- **notes**: OPEN / future-work prose

### L6693: `OPEN-MEM-001`
- **なぜ meta**: プロセス
- **notes**: OPEN / future-work prose

### L6703: `OPEN-MOD-001`
- **なぜ meta**: プロセス
- **notes**: OPEN / future-work prose

### L6711: 将来拡張
- **なぜ meta**: プロセス
- **notes**: OPEN / future-work prose

### L6724: 確立された基本原則
- **なぜ meta**: プロセス
- **notes**: BND status/principles prose

### L6858: 0.2 本項目が定めないもの
- **なぜ meta**: プロセス
- **notes**: 定めないもの=OPEN移管（意図的）

### L7374: 8. 実行可能artifactを生成する
- **なぜ meta**: プロセス
- **notes**: artifact生成は実行パイプライン側

### L7980: 22.7 下位項目
- **なぜ meta**: プロセス
- **notes**: 下位OPEN一覧

### L8016: 22.8 最終状態
- **なぜ meta**: 完成判定の文章
- **notes**: 最終RESOLVED宣言（追跡用）

### L8049: 状態
- **なぜ meta**: プロセス
- **notes**: TYP-DYN resolved-in-spec; impl is Dynamic unify stub

### L10197: 関連する後続設計課題
- **なぜ meta**: プロセス
- **notes**: spec process / OPEN pointer

### L10201: `OPEN-TYP-002`
- **なぜ meta**: プロセス
- **notes**: OPEN / principles under TYP-DYN

### L10210: `OPEN-SYN-002`／`LIT-001`
- **なぜ meta**: プロセス
- **notes**: OPEN / principles under TYP-DYN

### L10225: `OPEN-ERR-001`
- **なぜ meta**: プロセス
- **notes**: OPEN / principles under TYP-DYN

### L10233: `OPEN-MOD-001`
- **なぜ meta**: プロセス
- **notes**: OPEN / principles under TYP-DYN

### L10240: `OPEN-KER-001`
- **なぜ meta**: プロセス
- **notes**: OPEN / principles under TYP-DYN

### L10248: 将来拡張
- **なぜ meta**: プロセス
- **notes**: spec process / OPEN pointer

### L10262: 確立された基本原則
- **なぜ meta**: プロセス
- **notes**: spec process / OPEN pointer

### L13037: 関連する後続設計課題
- **なぜ meta**: プロセス
- **notes**: spec process / OPEN pointer

### L13039: `OPEN-SYN-002`
- **なぜ meta**: プロセス
- **notes**: OPEN / principles under TYP-ALG

### L13053: `OPEN-DAT-001`
- **なぜ meta**: プロセス
- **notes**: OPEN / principles under TYP-ALG

### L13063: `OPEN-EFF-001`の後続仕様
- **なぜ meta**: プロセス
- **notes**: OPEN / principles under TYP-ALG

### L13071: `OPEN-MOD-001`
- **なぜ meta**: プロセス
- **notes**: OPEN / principles under TYP-ALG

### L13079: `OPEN-ERR-001`
- **なぜ meta**: プロセス
- **notes**: OPEN / principles under TYP-ALG

### L13087: 将来拡張
- **なぜ meta**: プロセス
- **notes**: spec process / OPEN pointer

### L13104: 確立された基本原則
- **なぜ meta**: プロセス
- **notes**: spec process / OPEN pointer

### L14413: DD-001 決定概要
- **なぜ meta**: プロセス
- **notes**: decision overview

### L14414: DD-001.1 状態
- **なぜ meta**: 完成判定の文章
- **notes**: RESOLVED scope list; impl covers outer+import subset only

### L14452: 0. 適用範囲
- **なぜ meta**: プロセス
- **notes**: scope framing

### L14453: 0.1 本項目が定めるもの
- **なぜ meta**: プロセス
- **notes**: lists MOD surface; many items deferred

### L14531: 1.4 一ファイル一モジュールとの違い
- **なぜ meta**: プロセス
- **notes**: design note vs one-file-one-module

### L14679: 4.4 新しいscope
- **なぜ meta**: プロセス
- **notes**: new scope rule; nested modules absent

### L14843: 7.3 Source spelling
- **なぜ meta**: プロセス
- **notes**: source spelling vs identity

### L15513: 21.2 非公開情報
- **なぜ meta**: プロセス
- **notes**: privacy rules; no metadata emitter yet

### L15540: 21.5 Hashに含めないもの
- **なぜ meta**: プロセス
- **notes**: hash exclusions; no hasher

### L15719: 22. 移管先OPEN・下位項目・状態
- **なぜ meta**: プロセス
- **notes**: OPEN transfer table

### L15792: 22.7 下位項目
- **なぜ meta**: プロセス
- **notes**: sub-item index

### L15831: 22.8 最終状態
- **なぜ meta**: 完成判定の文章
- **notes**: MOD final-state prose; impl = outer+import skeleton

### L15856: DD-001 決定概要
- **なぜ meta**: 用語定義
- **notes**: PKG glossary/decision prose; no direct impl obligation

### L15857: DD-001.1 状態
- **なぜ meta**: 用語定義
- **notes**: PKG glossary/decision prose; no direct impl obligation

### L15878: DD-001.2 中心的な決定
- **なぜ meta**: 用語定義
- **notes**: PKG glossary/decision prose; no direct impl obligation

### L15903: 0. 適用範囲
- **なぜ meta**: 用語定義
- **notes**: PKG glossary/decision prose; no direct impl obligation

### L15904: 0.1 本項目が定めるもの
- **なぜ meta**: 用語定義
- **notes**: PKG glossary/decision prose; no direct impl obligation

### L15934: 0.2 本項目が直接定めないもの
- **なぜ meta**: 用語定義
- **notes**: PKG glossary/decision prose; no direct impl obligation

### L15954: 1. パッケージ
- **なぜ meta**: 用語定義
- **notes**: PKG glossary/decision prose; no direct impl obligation

### L15955: 1.1 定義
- **なぜ meta**: 用語定義
- **notes**: PKG glossary/decision prose; no direct impl obligation

### L15969: 1.2 モジュールとの違い
- **なぜ meta**: 用語定義
- **notes**: PKG glossary/decision prose; no direct impl obligation

### L15979: 1.3 パッケージの種類
- **なぜ meta**: 用語定義
- **notes**: PKG glossary/decision prose; no direct impl obligation

### L16814: 20.1 定義
- **なぜ meta**: 用語定義
- **notes**: PKG glossary/decision prose; no direct impl obligation

### L17273: 22.9 最終状態
- **なぜ meta**: 完成判定の文章
- **notes**: PKG final-state / resolved declaration prose

### L17356: DD-001 決定概要
- **なぜ meta**: プロセス
- **notes**: decision overview

### L17357: DD-001.1 状態
- **なぜ meta**: プロセス
- **notes**: status prose

### L17375: DD-001.2 位置付け
- **なぜ meta**: プロセス
- **notes**: positioning vs GUI

### L17400: 0. 適用範囲
- **なぜ meta**: プロセス
- **notes**: scope

### L18525: 22.3 信頼境界
- **なぜ meta**: プロセス
- **notes**: trust boundary prose

### L18546: 23.3 Effectの暫定分類
- **なぜ meta**: プロセス
- **notes**: provisional effect taxonomy

### L18776: 28. 移管先OPEN
- **なぜ meta**: プロセス
- **notes**: OPEN transfer

### L18862: 概要・状態
- **なぜ meta**: プロセス
- **notes**: IR-001 overview/status (単一万能IR拒否確定; schema暫定)

### L18962: 不変条件
- **なぜ meta**: プロセス
- **notes**: IR invariants prose

### L18972: メタ理論・反例
- **なぜ meta**: メタ理論
- **notes**: IR metatheory / CE notes

### L18994: DD-001 決定概要
- **なぜ meta**: プロセス
- **notes**: ERR design principle / guideline prose

### L18995: DD-001.1 状態
- **なぜ meta**: プロセス
- **notes**: ERR design principle / guideline prose

### L19015: DD-001.2 既存仕様との関係
- **なぜ meta**: プロセス
- **notes**: ERR design principle / guideline prose

### L19041: 0. 設計原則
- **なぜ meta**: プロセス
- **notes**: ERR design principle / guideline prose

### L19042: 0.1 失敗を一種類に統合しない
- **なぜ meta**: プロセス
- **notes**: ERR design principle / guideline prose

### L19063: 0.2 判断基準
- **なぜ meta**: プロセス
- **notes**: ERR design principle / guideline prose

### L19076: 0.3 公開APIと内部実装
- **なぜ meta**: プロセス
- **notes**: ERR design principle / guideline prose

### L19360: 7.1 基本指針
- **なぜ meta**: プロセス
- **notes**: ERR design principle / guideline prose

### L19401: 7.3 位置付け
- **なぜ meta**: プロセス
- **notes**: ERR design principle / guideline prose

### L19479: 9.2 Failureを認める場合
- **なぜ meta**: プロセス
- **notes**: ERR design principle / guideline prose

### L19602: 14.4 継続状態
- **なぜ meta**: プロセス
- **notes**: ERR design principle / guideline prose

### L19750: 18.1 分類
- **なぜ meta**: プロセス
- **notes**: ERR design principle / guideline prose

### L19886: 22. 個別事例の分類
- **なぜ meta**: プロセス
- **notes**: ERR design principle / guideline prose

### L20450: 31. 最終状態
- **なぜ meta**: 完成判定の文章
- **notes**: ERR final-state prose

### L20477: DD-001 決定概要
- **なぜ meta**: 用語定義
- **notes**: MEM glossary / policy prose

### L20478: DD-001.1 状態
- **なぜ meta**: 用語定義
- **notes**: MEM glossary / policy prose

### L20496: DD-001.2 既存仕様との関係
- **なぜ meta**: 用語定義
- **notes**: MEM glossary / policy prose

### L20515: DD-001.3 中心的な決定
- **なぜ meta**: 用語定義
- **notes**: MEM glossary / policy prose

### L20530: 0. 用語
- **なぜ meta**: 用語定義
- **notes**: MEM glossary / policy prose

### L20531: 0.1 Perceus
- **なぜ meta**: 用語定義
- **notes**: MEM glossary / policy prose

### L20549: 0.2 自動メモリ管理
- **なぜ meta**: 用語定義
- **notes**: MEM glossary / policy prose

### L20562: 0.3 Resource
- **なぜ meta**: 用語定義
- **notes**: MEM glossary / policy prose

### L20976: 10.3 Perceusとの関係
- **なぜ meta**: 用語定義
- **notes**: MEM glossary / policy prose

### L21958: 35. 最終状態
- **なぜ meta**: 完成判定の文章
- **notes**: MEM final-state prose

### L21990: 状態
- **なぜ meta**: プロセス
- **notes**: ASY status: 未決定 (thread/task/async not specified)

### L22014: 概要・状態
- **なぜ meta**: プロセス
- **notes**: TST overview: product testing確定; surface syntax 未決定

### L22045: 既存実装
- **なぜ meta**: プロセス
- **notes**: describes current Rust tests empirically

### L22067: メタ理論
- **なぜ meta**: メタ理論
- **notes**: tests ≠ proofs metatheory note

### L22072: 構文の統合仕様
- **なぜ meta**: プロセス
- **notes**: integrative syntax chapter; incomplete by design

### L22146: 静的意味論の統合仕様
- **なぜ meta**: プロセス
- **notes**: integrative statics chapter

### L22241: 動的意味論の統合仕様
- **なぜ meta**: プロセス
- **notes**: integrative dynamics chapter

### L22287: 16.5 観測可能な振る舞い
- **なぜ meta**: プロセス
- **notes**: observational behavior framing

### L22300: エラーと停止状態
- **なぜ meta**: プロセス
- **notes**: error/stuck taxonomy prose

### L22319: 機能間の相互作用
- **なぜ meta**: プロセス
- **notes**: cross-feature interaction notes

### L22339: メタ理論上の性質
- **なぜ meta**: メタ理論
- **notes**: Progress/Preservation goals; proofs absent

### L22440: 仕様と実装の対応
- **なぜ meta**: プロセス
- **notes**: stage correspondence table

### L22474: テスト計画
- **なぜ meta**: プロセス
- **notes**: test plan chapter

### L22559: 完成判定基準
- **なぜ meta**: 完成判定の文章
- **notes**: completion criteria process
