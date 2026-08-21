//! App registry: a fixed table of bundles. Index = app id for the ABI.

use crate::bundle::{Bundle, BundleError};
use crate::limits::MAX_APPS;
use heapless::Vec;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RegistryError {
    Full,
    Bundle(BundleError),
    DuplicateId,
}

#[derive(Default)]
pub struct Registry {
    apps: Vec<Bundle<'static>, MAX_APPS>,
}

impl Registry {
    pub const fn new() -> Self {
        Self { apps: Vec::new() }
    }

    pub fn add(&mut self, bytes: &'static [u8]) -> Result<usize, RegistryError> {
        let bundle = Bundle::parse(bytes).map_err(RegistryError::Bundle)?;
        if self.apps.iter().any(|b| b.id() == bundle.id()) {
            return Err(RegistryError::DuplicateId);
        }
        self.apps.push(bundle).map_err(|_| RegistryError::Full)?;
        Ok(self.apps.len() - 1)
    }

    pub fn len(&self) -> usize {
        self.apps.len()
    }

    pub fn is_empty(&self) -> bool {
        self.apps.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<Bundle<'static>> {
        self.apps.get(index).copied()
    }

    pub fn find(&self, id: &str) -> Option<usize> {
        self.apps.iter().position(|b| b.id() == id)
    }

    pub fn iter(&self) -> impl Iterator<Item = Bundle<'static>> + '_ {
        self.apps.iter().copied()
    }
}
