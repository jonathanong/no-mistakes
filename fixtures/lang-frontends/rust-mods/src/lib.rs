pub mod aaa;
pub mod mail;

pub use crate::mail::Welcome;

pub fn send() {
    Welcome::emit();
}
