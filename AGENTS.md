# GPUI Knowledge Base (v0.159.5)

Luminara Engine uses GPUI `v0.159.5`. This document summarizes the API specifications and troubleshooting steps discovered during the migration and debugging process.

## API Specifications

### 1. Render Trait
The `Render` trait signature requires the `render` method to return `impl IntoElement`.
```rust
impl Render for MyView {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div().child("Hello World")
    }
}
```

### 2. Element Trait
Types implementing `gpui::Element` must satisfy several requirements at this version:
- **`IntoElement` Supertrait**: Any type implementing `Element` must also implement `IntoElement`.
- **`id()` method**: Must return a `ElementId`.
- **`request_layout`**: Use `cx.request_layout(style, children)`.

Example:
```rust
impl IntoElement for MyElement {
    type Element = Self;
    fn into_element(self) -> Self::Element { self }
}

impl Element for MyElement {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> { None }
    
    fn request_layout(&mut self, _id: Option<&GlobalElementId>, cx: &mut WindowContext) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        // ...
        (cx.request_layout(&style, None), ())
    }
    // ...
}
```

### 3. VisualContext
The `new_view` method is defined on the `VisualContext` trait. To call `cx.new_view(...)` within a `ViewContext`, `VisualContext` must be in scope.
```rust
use gpui::VisualContext as _;

cx.new_view(|cx| MyView::new(cx))
```

## Troubleshooting & Pitfalls

### Cargo.toml Dependency Order
**CRITICAL**: In `Cargo.toml`, general dependencies MUST be placed before any `[target.'cfg(...)'.dependencies]` sections. 
If placed after, the TOML parser may treat them as part of the last target section, causing "unresolved crate" errors in other targets.

### Macro Export Paths
GPUI macros (like `actions!`) and some `luminara_core` macros (like `impl_component!`) are exported at the **crate root** when using `#[macro_export]`.
- Correct: `use luminara_core::impl_component;`
- Incorrect: `use luminara_core::component::impl_component;`

### Missing Methods in v0.159.5
Some methods available in newer Zed/GPUI versions do not exist or have different names:
- `overflow_y_scroll()` -> Use `overflow_hidden()` or manage via styles if available.
- `when_some()` -> Use standard Rust `if let Some(x) = opt { ... }`.

## Git Dependency configuration
In `Cargo.toml`, ensure the tag/branch corresponds exactly to the released version if using git dependencies.
```toml
gpui = { git = "https://github.com/zed-industries/zed", tag = "v0.159.5" }
```

## バックグラウンドスレッドからUIへの通知ブリッジ (v0.159.5)

### 問題
`GetAsyncKeyState` 等を使用するバックグラウンドスレッドから `EditorState` を更新しても、GPUIのメインスレッド（描画ループ）がそれを検知できず、UIが即座に更新されません。

### 解決策: ジェネレーションカウンタ方式
状態（State）を `AtomicU64` の世代カウンタと一緒に保持し、描画スレッドでポーリングを行います。

1. **Shared State 構造**:
   ```rust
   pub struct SharedEditorState {
       state: Arc<RwLock<EditorState>>,
       generation: Arc<AtomicU64>,
   }
   ```
2. **更新側**: `state.write()` を行う際に `generation.fetch_add(1, Ordering::Release)` を呼び出します。
3. **描画側 (`cx.spawn`)**:
   `window` の初期化時に以下のようなループを開始します。
   ```rust
   cx.spawn(|this, mut cx| async move {
       let mut last_gen = 0;
       loop {
           cx.background_executor().timer(Duration::from_millis(16)).await;
           let current_gen = generation.load(Ordering::Acquire);
           if current_gen != last_gen {
               last_gen = current_gen;
               this.update(&mut cx, |this, cx| cx.notify());
           }
       }
   }).detach();
   ```

## SVGアセットの読み込み (v0.159.5)

### 問題
`svg().path("icons/search.svg")` を呼び出しても、デフォルトではアセットが見つからず表示されません。

### 解決策: カスタム AssetSource の実装
`AssetSource` トレイトを実装した構造体を作成し、`Application` に登録します。

1. **実装**: `load` と `list` を実装し、ファイルシステムから `Cow<'static, [u8]>` を返します。
2. **登録**:
   ```rust
   let asset_source = EditorAssetSource::new(PathBuf::from("assets"));
   GpuiApp::new()
       .with_assets(asset_source)
       .run(move |cx| { ... });
   ```
3. **注意**: `AssetSource` を登録した場合、`svg().path()` に渡すパスは `AssetSource` のルートディレクトリからの相対パスになります。`assets/icons/search.svg` ではなく `icons/search.svg` と指定します。

## アクティビティバーのレイアウト設計

VS Code風のレイアウト（下部に設定ボタン等を配置）を実現するには、`flex_col()` 内で `flex_1()` のスペーサーを挿入して、下部アイテムを `margin-top: auto` 相当で押し下げます。

```rust
div()
    .flex()
    .flex_col()
    .size_full()
    .child(top_items) // メイン項目
    .child(div().flex_1()) // スペーサー
    .child(bottom_items) // 設定・アカウント等
```

## 3Dビューポートの実装 (v0.159.5)

### 問題
GPUI v0.159.5では外部WGPUテクスチャを直接描画できないため、ビューーポートには3Dシーンが表示されません。

### 解決策: 別ウィンドウ方式

`crates/luminara_editor/examples/hybrid_editor.rs` で実証されているように、3Dビューーポートを別ウィンドウとして実装します：

1. **winitで別ウィンドウ作成**: 3Dレンダリング用の別ウィンドウを作成
2. **WGPUコンテキスト共有**: 同じWGPUデバイスを使用
3. **カメラ状態同期**: `ViewportWindowState` を使用してカメラ情報を共有

```rust
use luminara_editor::core::viewport_window::{ViewportWindowState, create_viewport_state};

// 共有状態を作成
let viewport_state = create_viewport_state();

// GPUIエディタスレッドからカメラ更新
viewport_state.write().set_camera(position, target, up, fov);

// 別ウィンドウスレッドでカメラ読み取り
let camera = viewport_state.read();
// レンダリング...
```

### ビューポート状態のエクスポート

`crates/luminara_editor/src/core/viewport_window.rs` で `ViewportWindowState` をエクスポートしています。


