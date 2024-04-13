use anyhow::Result;
use keyring::Entry;

#[derive(Clone)]
pub struct Profile {
    pub name: String,
    pub key: String,
}

impl Profile {
    pub fn load(name: String) -> Result<Self> {
        let key = Entry::new("air", &name)?.get_password()?;
        Ok(Self { name, key })
    }

    pub fn list() -> Result<Vec<Self>> {
        unimplemented!("missing list functionality in keyring-rs dependency")
    }

    pub fn delete(self) -> Result<()> {
        Ok(Entry::new("air", &self.name)?.delete_password()?)
    }

    pub fn save(&self) -> Result<()> {
        Ok(Entry::new("air", &self.name)?.set_password(&self.key)?)
    }
}
