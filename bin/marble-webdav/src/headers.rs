use http::HeaderName;
use once_cell::sync::Lazy;

// Missing WebDAV headers
pub static DESTINATION: Lazy<HeaderName> = Lazy::new(|| HeaderName::from_static("destination"));
pub static DAV: Lazy<HeaderName> = Lazy::new(|| HeaderName::from_static("dav"));
pub static DEPTH: Lazy<HeaderName> = Lazy::new(|| HeaderName::from_static("depth"));
pub static LOCK_TOKEN: Lazy<HeaderName> = Lazy::new(|| HeaderName::from_static("lock-token"));
pub static TIMEOUT: Lazy<HeaderName> = Lazy::new(|| HeaderName::from_static("timeout"));
pub static OVERWRITE: Lazy<HeaderName> = Lazy::new(|| HeaderName::from_static("overwrite"));