pub mod mail;

use crate::mail::Welcome;

pub fn send() {
    Welcome::emit();
}
