use luminara_core::Resource;
use std::collections::{HashMap, VecDeque};

pub struct DiagnosticEntry {
    pub name: String,
    pub value: f64,
    pub history: VecDeque<f64>,
    pub max_history: usize,
}

impl DiagnosticEntry {
    pub fn new(name: String, max_history: usize) -> Self {
        Self {
            name,
            value: 0.0,
            history: VecDeque::with_capacity(max_history),
            max_history,
        }
    }

    pub fn add(&mut self, value: f64) {
        self.value = value;
        if self.history.len() >= self.max_history {
            self.history.pop_front();
        }
        self.history.push_back(value);
    }

    pub fn average(&self) -> f64 {
        if self.history.is_empty() {
            return 0.0;
        }
        self.history.iter().sum::<f64>() / self.history.len() as f64
    }
}

pub struct Diagnostics {
    pub entries: HashMap<String, DiagnosticEntry>,
}

impl Resource for Diagnostics {}

impl Diagnostics {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn add(&mut self, name: &str, value: f64) {
        let entry = self
            .entries
            .entry(name.to_string())
            .or_insert_with(|| DiagnosticEntry::new(name.to_string(), 120));
        entry.add(value);
    }

    pub fn get(&self, name: &str) -> Option<&DiagnosticEntry> {
        self.entries.get(name)
    }

    pub fn get_average(&self, name: &str) -> Option<f64> {
        self.entries.get(name).map(|e| e.average())
    }
}

impl Default for Diagnostics {
    fn default() -> Self {
        Self::new()
    }
}
