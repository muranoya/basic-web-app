SPAでWebアプリケーションを開発するときのテンプレートプログラム。

# サーバサイド

以下の想定。

* 言語: Rust
* HTMLテンプレートエンジン: maud
* Webフレームワーク: Axum
* SQL: sqlx
* データベース: SQLite3 (ホビー用途だがそこそこのリクエストには耐えられるはず)

## ディレクトリ構成

```
src/auth: ログイン・ログアウトなどの認証周りのコード。
src/csrf: CSRFトークン関係のコード。
src/models: データベースのテーブルデータを射影するRustの構造体。
src/repositories: データベース操作に関するコード。
```

# フロントエンド

以下の想定。

* 言語: TypeScript
* フレームワーク: React
* UIコンポーネント: MUI
* バンドラー: vite
* パッケージマネージャ: pnpm

## ディレクトリ構成

```
frontend/src/api: サーバにリクエストを送信するコード。
frontend/src/components: Reactコンポーネント。このディレクトリのコードはUIに関する状態制御のみにする。
frontend/src/pages: ReactRouterでルーティングする先のコンポーネント。実際のサーバにリクエストを送信したりする。
frontend/src/stores: jotaiによる状態管理に関するコード。
frontend/src/types: interfaceやtypeなどの定義。
frontend/src/utils: ユーティリティコード。
```

# セッション管理

ログインするとCookie (HttpOnly, Secure, SameSite: Lax) にセッションキーを書き込む。

以後、このセッションキーをもとにログインユーザを特定する。

# サーバサイドの基本挙動

`src/main.rs` のindex関数でHTMLを返す。このHTMLを元に `frontend/src/pages` のReactコンポーネントをマウントして描画する。

viteでビルドしたバンドルJSファイルについては、 `frontend/dist/assets` ディレクトリに生成される。バンドルファイル名は `frontend/dist/.vite/manifest.json` を参照して把握できる。

ログインしている場合、Headタグ内のmetaタグにCSRFトークンを記載しておき、POST/PUT/DELETEなどのサーバサイドの状態変更を伴うリクエストを送る時はCSRFトークンをリクエストに付与して送信することとする。
