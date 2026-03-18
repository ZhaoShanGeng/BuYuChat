use std::sync::Arc;

use crate::error::{AppError, Result};

pub struct KeyringService {
    service_name: String,
    #[cfg(test)]
    memory: std::sync::Mutex<std::collections::HashMap<String, String>>,
}

impl KeyringService {
    pub fn new(service_name: impl Into<String>) -> Arc<Self> {
        Arc::new(Self {
            service_name: service_name.into(),
            #[cfg(test)]
            memory: std::sync::Mutex::new(std::collections::HashMap::new()),
        })
    }

    pub fn save(&self, key_id: &str, value: &str) -> Result<()> {
        #[cfg(test)]
        {
            self.memory
                .lock()
                .map_err(|_| AppError::Other("test keyring mutex poisoned".to_string()))?
                .insert(key_id.to_string(), value.to_string());
            return Ok(());
        }

        #[cfg(not(test))]
        {
            self.entry(key_id)?.set_password(value)?;
            Ok(())
        }
    }

    pub fn get(&self, key_id: &str) -> Result<String> {
        #[cfg(test)]
        {
            return self
                .memory
                .lock()
                .map_err(|_| AppError::Other("test keyring mutex poisoned".to_string()))?
                .get(key_id)
                .cloned()
                .ok_or(keyring::Error::NoEntry)
                .map_err(AppError::Keyring);
        }

        #[cfg(not(test))]
        {
            Ok(self.entry(key_id)?.get_password()?)
        }
    }

    pub fn get_optional(&self, key_id: &str) -> Result<Option<String>> {
        #[cfg(test)]
        {
            return Ok(self
                .memory
                .lock()
                .map_err(|_| AppError::Other("test keyring mutex poisoned".to_string()))?
                .get(key_id)
                .cloned());
        }

        #[cfg(not(test))]
        {
            match self.entry(key_id)?.get_password() {
                Ok(value) => Ok(Some(value)),
                Err(keyring::Error::NoEntry) => Ok(None),
                Err(err) => Err(AppError::Keyring(err)),
            }
        }
    }

    pub fn contains(&self, key_id: &str) -> Result<bool> {
        Ok(self.get_optional(key_id)?.is_some())
    }

    pub fn delete(&self, key_id: &str) -> Result<()> {
        #[cfg(test)]
        {
            self.memory
                .lock()
                .map_err(|_| AppError::Other("test keyring mutex poisoned".to_string()))?
                .remove(key_id);
            return Ok(());
        }

        #[cfg(not(test))]
        {
            match self.entry(key_id)?.delete_credential() {
                Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
                Err(err) => Err(AppError::Keyring(err)),
            }
        }
    }

    fn entry(&self, key_id: &str) -> Result<keyring::Entry> {
        Ok(keyring::Entry::new(&self.service_name, key_id)?)
    }
}
