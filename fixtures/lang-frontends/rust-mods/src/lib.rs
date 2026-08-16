pub mod aaa;
pub mod mail;

use crate::{aaa, mail as delivery};
use crate::aaa::helper::Item;
pub use crate::mail::Welcome;
use self::mail::Welcome as LocalWelcome;

pub fn send() {
    Welcome::emit();
}
