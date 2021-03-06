// Copyright 2017 Parity Technologies (UK) Ltd.
// This file is part of Substrate Demo.

// Substrate Demo is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate Demo is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate Demo.  If not, see <http://www.gnu.org/licenses/>.

//! Timestamp manager: just handles the current timestamp.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg_attr(any(feature = "std", test), macro_use)]
extern crate substrate_runtime_std as rstd;

#[macro_use]
extern crate substrate_runtime_support as runtime_support;

#[cfg(any(feature = "std", test))]
extern crate substrate_runtime_io as runtime_io;

#[cfg(test)]
extern crate substrate_primitives;
extern crate substrate_runtime_primitives as runtime_primitives;
extern crate substrate_runtime_system as system;
extern crate substrate_codec as codec;

use runtime_support::{StorageValue, Parameter};
use runtime_primitives::traits::{HasPublicAux, Executable, MaybeEmpty};

pub trait Trait: HasPublicAux + system::Trait {
	type Value: Parameter + Default;
}

decl_module! {
	pub struct Module<T: Trait>;
	pub enum Call where aux: T::PublicAux {
		fn set(aux, now: T::Value) = 0;
	}
}

decl_storage! {
	trait Store for Module<T: Trait>;
	pub Now get(now): b"tim:val" => required T::Value;

	// Did the timestamp get updated in this block?
	DidUpdate: b"tim:did" => default bool;
}

impl<T: Trait> Module<T> {
	pub fn get() -> T::Value {
		<Self as Store>::Now::get()
	}

	/// Set the current time.
	fn set(aux: &T::PublicAux, now: T::Value) {
		assert!(aux.is_empty());
		assert!(!<Self as Store>::DidUpdate::exists(), "Timestamp must be updated only once in the block");
		assert!(<system::Module<T>>::extrinsic_index() == 0, "Timestamp must be first extrinsic in the block");
		<Self as Store>::Now::put(now);
		<Self as Store>::DidUpdate::put(true);
	}
}

impl<T: Trait> Executable for Module<T> {
	fn execute() {
		assert!(<Self as Store>::DidUpdate::take(), "Timestamp must be updated once in the block");
	}
}

#[cfg(any(feature = "std", test))]
#[derive(Default)]
pub struct GenesisConfig<T: Trait> {
	pub now: T::Value,
}

#[cfg(any(feature = "std", test))]
impl<T: Trait> runtime_primitives::BuildExternalities for GenesisConfig<T>
{
	fn build_externalities(self) -> runtime_primitives::BuiltExternalities {
		use runtime_io::twox_128;
		use codec::Slicable;
		map![
			twox_128(<Now<T>>::key()).to_vec() => self.now.encode()
		]
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	use runtime_io::with_externalities;
	use runtime_support::storage::StorageValue;
	use substrate_primitives::H256;
	use runtime_primitives::BuildExternalities;
	use runtime_primitives::traits::{HasPublicAux};
	use runtime_primitives::testing::{Digest, Header};

	pub struct Test;
	impl HasPublicAux for Test {
		type PublicAux = u64;
	}
	impl system::Trait for Test {
		type Index = u64;
		type BlockNumber = u64;
		type Hash = H256;
		type Hashing = runtime_io::BlakeTwo256;
		type Digest = Digest;
		type AccountId = u64;
		type Header = Header;
	}
	impl Trait for Test {
		type Value = u64;
	}
	type Timestamp = Module<Test>;

	#[test]
	fn timestamp_works() {
		let mut t = system::GenesisConfig::<Test>::default().build_externalities();
		t.extend(GenesisConfig::<Test> { now: 42 }.build_externalities());

		with_externalities(&mut t, || {
			assert_eq!(<Timestamp as Store>::Now::get(), 42);
			Timestamp::aux_dispatch(Call::set(69), &0);
			assert_eq!(Timestamp::now(), 69);
		});
	}
}
