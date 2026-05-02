use crate::common::error::BlazeResult;
use rc4::{Key, KeyInit, Rc4, StreamCipher};

/// RC4 encryption/decryption for Blaze protocol
pub struct BlazeCrypto {
    cipher: Rc4<rc4::consts::U16>,
}

impl BlazeCrypto {
    /// Create new crypto instance with key
    pub fn new(key: &[u8]) -> Self {
        let key = Key::from_slice(key);
        let cipher = Rc4::new(key);
        Self { cipher }
    }

    /// Encrypt data in-place
    pub fn encrypt(&mut self, data: &mut [u8]) -> BlazeResult<()> {
        self.cipher.apply_keystream(data);
        Ok(())
    }

    /// Decrypt data in-place
    pub fn decrypt(&mut self, data: &mut [u8]) -> BlazeResult<()> {
        self.cipher.apply_keystream(data);
        Ok(())
    }

    /// Encrypt data and return new vector
    pub fn encrypt_copy(&mut self, data: &[u8]) -> BlazeResult<Vec<u8>> {
        let mut result = data.to_vec();
        self.encrypt(&mut result)?;
        Ok(result)
    }

    /// Decrypt data and return new vector
    pub fn decrypt_copy(&mut self, data: &[u8]) -> BlazeResult<Vec<u8>> {
        let mut result = data.to_vec();
        self.decrypt(&mut result)?;
        Ok(result)
    }
}

/// Session state for a client connection
pub struct SessionState {
    pub crypto_enabled: bool,
    pub c_in: BlazeCrypto,  // Client to server encryption
    pub c_out: BlazeCrypto, // Server to client encryption
    pub update_network_info_count: u32, // Track number of updateNetworkInfo calls
    /// UI registry id for this Blaze TCP connection (`None` if not registered)
    pub blaze_session_id: Option<u64>,
    /// Last value pushed to `blaze_sessions::set_crypto_enabled` (avoids registry lock every read).
    pub last_registry_crypto: Option<bool>,
}

impl SessionState {
    /// Create new session state
    pub fn new() -> Self {
        // Default key - in real implementation, this would be negotiated
        let default_key = b"default_key_16by"; // Exactly 16 bytes

        Self {
            crypto_enabled: false, // Start with crypto disabled
            c_in: BlazeCrypto::new(default_key),
            c_out: BlazeCrypto::new(default_key),
            update_network_info_count: 0,
            blaze_session_id: None,
            last_registry_crypto: None,
        }
    }

    /// Enable encryption with new keys
    pub fn enable_crypto(&mut self, client_key: &[u8], server_key: &[u8]) {
        self.crypto_enabled = true;
        self.c_in = BlazeCrypto::new(client_key);
        self.c_out = BlazeCrypto::new(server_key);
    }
}
