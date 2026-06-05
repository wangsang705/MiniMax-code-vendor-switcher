use keyring::Entry;

#[derive(Debug, thiserror::Error)]
pub enum KeyringError {
    #[error("keyring: {0}")]
    Keyring(#[from] keyring::Error),
}

pub fn set_key(service: &str, account: &str, value: &str) -> Result<(), KeyringError> {
    let entry = Entry::new(service, account)?;
    entry.set_password(value)?;
    Ok(())
}

pub fn get_key(service: &str, account: &str) -> Result<String, KeyringError> {
    let entry = Entry::new(service, account)?;
    Ok(entry.get_password()?)
}

pub fn delete_key(service: &str, account: &str) -> Result<(), KeyringError> {
    let entry = Entry::new(service, account)?;
    entry.delete_password()?;
    Ok(())
}
