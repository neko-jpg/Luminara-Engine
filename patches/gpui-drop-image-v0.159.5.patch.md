# GPUI v0.159.5 — `drop_image` バックポートパッチ (アーカイブ)

> **⚠️ このパッチは不要になりました**
>
> GPUI v0.164.2 へのアップグレード (2024-12) により、`WindowContext::drop_image` が
> ネイティブで利用可能になりました。手動パッチの適用は不要です。
> このファイルは歴史的記録としてのみ残されています。

## 概要
GPUI v0.159.5 の `BladeAtlas` は画像テクスチャをキャッシュするが、エントリの削除 API がない。
本パッチは新しいリビジョン (commit `0341609`) から `PlatformAtlas::remove` と
`WindowContext::drop_image` をバックポートし、不要な atlas エントリを解放可能にする。

## 適用対象
Cargo git checkout: `~/.cargo/git/checkouts/zed-a70e2ad075855582/bcf6806/`

## パッチ内容 (4ファイル)

### 1. `crates/gpui/src/platform.rs` — PlatformAtlas トレイト

`PlatformAtlas` trait に `fn remove` を追加:

```rust
pub(crate) trait PlatformAtlas: Send + Sync {
    fn get_or_insert_with<'a>(
        &self,
        key: &AtlasKey,
        build: &mut dyn FnMut() -> Result<Option<(Size<DevicePixels>, Cow<'a, [u8]>)>>,
    ) -> Result<Option<AtlasTile>>;

    fn remove(&self, key: &AtlasKey);  // ← 追加
}
```

### 2. `crates/gpui/src/platform/blade/blade_atlas.rs` — BladeAtlas 実装

`impl PlatformAtlas for BladeAtlas` ブロック末尾に追加:

```rust
    fn remove(&self, key: &AtlasKey) {
        let mut lock = self.0.lock();
        if let Some(tile) = lock.tiles_by_key.remove(key) {
            let textures = &mut lock.storage[tile.texture_id.kind];
            if let Some(texture) = textures.get_mut(tile.texture_id.index as usize) {
                texture.allocator.deallocate(tile.tile_id.into());
            }
        }
    }
```

### 3. `crates/gpui/src/platform/mac/metal_atlas.rs` — MetalAtlas 実装

`impl PlatformAtlas for MetalAtlas` ブロック末尾に追加:

```rust
    fn remove(&self, key: &AtlasKey) {
        let mut lock = self.0.lock();
        if let Some(tile) = lock.tiles_by_key.remove(key) {
            let textures = match tile.texture_id.kind {
                crate::AtlasTextureKind::Monochrome => &mut lock.monochrome_textures,
                crate::AtlasTextureKind::Polychrome => &mut lock.polychrome_textures,
                crate::AtlasTextureKind::Path => &mut lock.path_textures,
            };
            if let Some(texture) = textures.get_mut(tile.texture_id.index as usize) {
                texture.allocator.deallocate(tile.tile_id.into());
            }
        }
    }
```

### 4. `crates/gpui/src/platform/test/window.rs` — TestAtlas 実装

`impl PlatformAtlas for TestAtlas` ブロック末尾に追加:

```rust
    fn remove(&self, key: &crate::AtlasKey) {
        let mut state = self.0.lock();
        state.tiles.remove(key);
    }
```

### 5. `crates/gpui/src/window.rs` — WindowContext::drop_image

`paint_image` メソッドの直後に追加:

```rust
    /// Removes a render image from the sprite atlas, freeing its GPU memory.
    /// Call this before dropping an `Arc<RenderImage>` to prevent atlas leaks.
    pub fn drop_image(&mut self, data: Arc<RenderImage>) -> Result<()> {
        for frame_index in 0..data.frame_count() {
            let params = RenderImageParams {
                image_id: data.id,
                frame_index,
            };
            self.window.sprite_atlas.remove(&params.clone().into());
        }
        Ok(())
    }
```

## 再適用手順

```powershell
# 1. cargo cache をクリーンした場合、checkout を復元
cargo update -p gpui  # Cargo.lock の gpui エントリを再取得

# 2. パッチを手動で適用 (上記 4 ファイルを編集)

# 3. gpui をクリーンビルドして変更を反映
cargo clean -p gpui
cargo check -p luminara_editor
```

## 使い方 (viewport.rs)

```rust
// 古い RenderImage を atlas から解放
if let Some(ImageSource::Render(old_image)) = self.current_image.take() {
    let _ = cx.drop_image(old_image);
}
// 新しい RenderImage を作成
self.create_render_image(frame);
```

## 恒久対策 (完了)
v0.164.2 にアップグレード済み。`drop_image` はネイティブ搭載されており、
手動パッチは不要となった。
