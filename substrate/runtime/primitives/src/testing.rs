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

//! Testing utilities.

use serde;
use codec::{Slicable, Input};
use runtime_support::AuxDispatchable;
use substrate_primitives::H256;
use traits::{self, Checkable, Applyable};

#[derive(Default, PartialEq, Eq, Clone, Serialize, Debug)]
pub struct Digest {
	pub logs: Vec<u64>,
}
impl Slicable for Digest {
	fn decode<I: Input>(input: &mut I) -> Option<Self> {
		Vec::<u64>::decode(input).map(|logs| Digest { logs })
	}
	fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
		self.logs.using_encoded(f)
	}
}
impl traits::Digest for Digest {
	type Item = u64;
	fn push(&mut self, item: Self::Item) {
		self.logs.push(item);
	}
}

#[derive(PartialEq, Eq, Clone, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Header {
	pub parent_hash: H256,
	pub number: u64,
	pub state_root: H256,
	pub extrinsics_root: H256,
	pub digest: Digest,
}
impl Slicable for Header {
	fn decode<I: Input>(input: &mut I) -> Option<Self> {
		Some(Header {
			parent_hash: Slicable::decode(input)?,
			number: Slicable::decode(input)?,
			state_root: Slicable::decode(input)?,
			extrinsics_root: Slicable::decode(input)?,
			digest: Slicable::decode(input)?,
		})
	}

	fn encode(&self) -> Vec<u8> {
		let mut v = Vec::new();
		self.parent_hash.using_encoded(|s| v.extend(s));
		self.number.using_encoded(|s| v.extend(s));
		self.state_root.using_encoded(|s| v.extend(s));
		self.extrinsics_root.using_encoded(|s| v.extend(s));
		self.digest.using_encoded(|s| v.extend(s));
		v
	}
}
impl traits::Header for Header {
	type Number = u64;
	type Hash = H256;
	type Digest = Digest;
	fn number(&self) -> &Self::Number { &self.number }
	fn extrinsics_root(&self) -> &Self::Hash { &self.extrinsics_root }
	fn state_root(&self) -> &Self::Hash { &self.state_root }
	fn parent_hash(&self) -> &Self::Hash { &self.parent_hash }
	fn digest(&self) -> &Self::Digest { &self.digest }
	fn new(
		number: Self::Number,
		extrinsics_root: Self::Hash,
		state_root: Self::Hash,
		parent_hash: Self::Hash,
		digest: Self::Digest
	) -> Self {
		Header {
			number, extrinsics_root: extrinsics_root, state_root, parent_hash, digest
		}
	}
}

#[derive(PartialEq, Eq, Clone, Serialize, Debug)]
pub struct Block<Xt: Slicable + Sized + serde::Serialize> {
	pub header: Header,
	pub extrinsics: Vec<Xt>,
}
impl<Xt: Slicable + Sized + serde::Serialize> Slicable for Block<Xt> {
	fn decode<I: Input>(input: &mut I) -> Option<Self> {
		Some(Block {
			header: Slicable::decode(input)?,
			extrinsics: Slicable::decode(input)?,
		})
	}
	fn encode(&self) -> Vec<u8> {
		let mut v: Vec<u8> = Vec::new();
		v.extend(self.header.encode());
		v.extend(self.extrinsics.encode());
		v
	}
}
impl<Xt: Slicable + Sized + serde::Serialize> traits::Block for Block<Xt> {
	type Extrinsic = Xt;
	type Header = Header;
	fn header(&self) -> &Self::Header {
		&self.header
	}
	fn extrinsics(&self) -> &[Self::Extrinsic] {
		&self.extrinsics[..]
	}
	fn deconstruct(self) -> (Self::Header, Vec<Self::Extrinsic>) {
		(self.header, self.extrinsics)
	}
}

#[derive(PartialEq, Eq, Clone, Serialize, Debug)]
pub struct TestXt<Call: AuxDispatchable + Slicable + Sized + serde::Serialize>(pub (u64, u64, Call));
impl<Call: AuxDispatchable + Slicable + Sized + serde::Serialize> Slicable for TestXt<Call> {
	fn decode<I: Input>(input: &mut I) -> Option<Self> {
		Some(TestXt(Slicable::decode(input)?))
	}
	fn encode(&self) -> Vec<u8> {
		self.0.encode()
	}
}
impl<Call: AuxDispatchable + Slicable + Sized + serde::Serialize> Checkable for TestXt<Call> {
	type Checked = Self;
	fn check(self) -> Result<Self, Self> { Ok(self) }
}
impl<Call: AuxDispatchable<Aux = u64> + Slicable + Sized + serde::Serialize> Applyable for TestXt<Call> {
	type AccountId = u64;
	type Index = u64;
	fn sender(&self) -> &u64 { &(self.0).0 }
	fn index(&self) -> &u64 { &(self.0).1 }
	fn apply(self) { (self.0).2.dispatch(&(self.0).0); }
}
