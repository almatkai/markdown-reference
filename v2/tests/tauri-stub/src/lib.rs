//! Test-only desktop boundary. No OS APIs are reachable.
#[derive(Clone)]
pub struct AppHandle;
pub trait Emitter {
    fn emit<T>(&self, _channel: &str, _payload: T) -> Result<(), String>;
}
impl Emitter for AppHandle {
    fn emit<T>(&self, _channel: &str, _payload: T) -> Result<(), String> {
        Err("Desktop events are forbidden in this harness".into())
    }
}
