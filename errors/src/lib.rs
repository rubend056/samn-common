#![cfg_attr(not(feature = "std"), no_std)]

pub trait Discriminant {
	fn discriminant(&self) -> u8;
	fn discriminant_max() -> u8;
}