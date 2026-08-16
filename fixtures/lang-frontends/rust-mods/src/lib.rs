pub mod aaa;
pub mod mail;

use crate::{aaa, mail as delivery};
pub use crate::mail::Welcome;

pub fn send() {
    Welcome::emit();
}
