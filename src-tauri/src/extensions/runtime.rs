use std::collections::HashMap;
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::domain::plugins::PluginDef;

#[derive(Debug, Default)]
pub struct PluginRuntime {
    plugins: RwLock<HashMap<String, PluginDef>>,
}

impl PluginRuntime {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn replace_all(&self, plugins: Vec<PluginDef>) {
        let mut guard = self.write_guard();
        guard.clear();
        for plugin in plugins {
            if plugin.enabled {
                guard.insert(plugin.id.clone(), plugin);
            }
        }
    }

    pub fn register_plugin(&self, plugin: PluginDef) {
        let mut guard = self.write_guard();
        if plugin.enabled {
            guard.insert(plugin.id.clone(), plugin);
        } else {
            guard.remove(&plugin.id);
        }
    }

    pub fn unregister_plugin(&self, plugin_id: &str) -> Option<PluginDef> {
        self.write_guard().remove(plugin_id)
    }

    pub fn list_plugins(&self) -> Vec<PluginDef> {
        let mut items = self.read_guard().values().cloned().collect::<Vec<_>>();
        items.sort_by(|left, right| {
            left.sort_order
                .cmp(&right.sort_order)
                .then_with(|| left.name.cmp(&right.name))
                .then_with(|| left.id.cmp(&right.id))
        });
        items
    }

    pub fn get_plugin(&self, plugin_id: &str) -> Option<PluginDef> {
        self.read_guard().get(plugin_id).cloned()
    }

    pub fn list_plugins_by_capability(&self, capability: &str) -> Vec<PluginDef> {
        self.list_plugins()
            .into_iter()
            .filter(|plugin| plugin.has_capability(capability))
            .collect()
    }

    fn read_guard(&self) -> RwLockReadGuard<'_, HashMap<String, PluginDef>> {
        match self.plugins.read() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    fn write_guard(&self) -> RwLockWriteGuard<'_, HashMap<String, PluginDef>> {
        match self.plugins.write() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }
}
