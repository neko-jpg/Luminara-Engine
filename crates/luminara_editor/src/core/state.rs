use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

/// Global editor state shared between UI and command system
///
/// This state is thread-safe and can be accessed from both
/// the UI thread and command handlers.
#[derive(Debug, Clone)]
pub struct EditorState {
    /// Global search visibility state
    pub global_search_visible: bool,
    /// Currently active box/tab
    pub active_box_index: usize,
    /// Window focus state
    pub window_focused: bool,
    // Add additional global state here (e.g., selected entity)
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            global_search_visible: false,
            active_box_index: 1, // Default to Scene Builder
            window_focused: true,
        }
    }
}

impl EditorState {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn toggle_global_search(&mut self) {
        self.global_search_visible = !self.global_search_visible;
        println!("Global Search toggled: {}", self.global_search_visible);
    }
    
    pub fn set_global_search_visible(&mut self, visible: bool) {
        self.global_search_visible = visible;
    }
    
    pub fn set_active_box(&mut self, index: usize) {
        self.active_box_index = index;
    }
}

/// Shared state wrapper that pairs EditorState with a lock-free generation counter.
#[derive(Clone)]
pub struct SharedEditorState {
    state: Arc<RwLock<EditorState>>,
    generation: Arc<AtomicU64>,
}

impl SharedEditorState {
    pub fn new(state: Arc<RwLock<EditorState>>) -> Self {
        Self {
            state,
            generation: Arc::new(AtomicU64::new(0)),
        }
    }
    
    pub fn state(&self) -> Arc<RwLock<EditorState>> {
        self.state.clone()
    }
    
    pub fn generation(&self) -> Arc<AtomicU64> {
        self.generation.clone()
    }
    
    pub fn read(&self) -> RwLockReadGuard<'_, EditorState> {
        self.state.read()
    }
    
    pub fn write(&self) -> SharedEditorStateWriteGuard<'_> {
        let guard = self.state.write();
        SharedEditorStateWriteGuard {
            guard,
            generation: &self.generation,
        }
    }
    
    pub fn toggle_global_search(&self) {
        self.state.write().toggle_global_search();
        self.generation.fetch_add(1, Ordering::Release);
    }
    
    pub fn set_global_search_visible(&self, visible: bool) {
        self.state.write().set_global_search_visible(visible);
        self.generation.fetch_add(1, Ordering::Release);
    }
    
    pub fn set_active_box(&self, index: usize) {
        self.state.write().set_active_box(index);
        self.generation.fetch_add(1, Ordering::Release);
    }
    
    pub fn current_generation(&self) -> u64 {
        self.generation.load(Ordering::Acquire)
    }
}

pub struct SharedEditorStateWriteGuard<'a> {
    guard: RwLockWriteGuard<'a, EditorState>,
    generation: &'a AtomicU64,
}

impl<'a> std::ops::Deref for SharedEditorStateWriteGuard<'a> {
    type Target = EditorState;
    fn deref(&self) -> &Self::Target {
        &self.guard
    }
}

impl<'a> std::ops::DerefMut for SharedEditorStateWriteGuard<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.guard
    }
}

impl<'a> Drop for SharedEditorStateWriteGuard<'a> {
    fn drop(&mut self) {
        self.generation.fetch_add(1, Ordering::Release);
    }
}
