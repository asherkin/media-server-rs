mod bridge;

use parking_lot::{Mutex, const_mutex};

// TODO: Figure out an error handling strategy once we have more errors.
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

static INIT_MUTEX: Mutex<bool> = const_mutex(false);

pub fn library_init() -> Result<()> {
    let mut is_init = INIT_MUTEX.lock();

    if *is_init {
        return Ok(());
    }

    *is_init = true;

    // TODO: Expose the logging config to consumers.
    bridge::logger_enable_log(true);
    bridge::logger_enable_debug(true);
    bridge::logger_enable_ultra_debug(false);

    bridge::openssl_class_init()?;

    // It is unfortunate that this is global state.
    bridge::dtls_connection_initialize()?;

    Ok(())
}