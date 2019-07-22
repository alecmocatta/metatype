//! Helper methods to determine whether a type is `TraitObject`, `Slice` or `Concrete`, and work with them respectively.
//!
//! # Examples
//!
//! ```
//! # use std::{any};
//! # use metatype::*;
//! assert_eq!(usize::METATYPE, MetaType::Concrete);
//! assert_eq!(any::Any::METATYPE, MetaType::TraitObject);
//! assert_eq!(<[u8]>::METATYPE, MetaType::Slice);
//!
//! let a: Box<usize> = Box::new(123);
//! assert_eq!((&*a).meta_type(), MetaType::Concrete);
//! let a: Box<dyn any::Any> = a;
//! assert_eq!((&*a).meta_type(), MetaType::TraitObject);
//!
//! let a = [123,456];
//! assert_eq!(a.meta_type(), MetaType::Concrete);
//! let a: &[i32] = &a;
//! assert_eq!(a.meta_type(), MetaType::Slice);
//!
//! let a: Box<dyn any::Any> = Box::new(123);
//! // https://github.com/rust-lang/rust/issues/50318
//! // let meta: TraitObject = (&*a).meta();
//! // println!("vtable: {:?}", meta.vtable);
//! ```
//!
//! # Note
//!
//! This currently requires Rust nightly for the `raw` and `specialization` features.

#![doc(html_root_url = "https://docs.rs/metatype/0.1.1")]
#![feature(raw, box_syntax, specialization)]
#![warn(
	missing_copy_implementations,
	missing_debug_implementations,
	missing_docs,
	// trivial_casts,
	trivial_numeric_casts,
	unused_import_braces,
	unused_qualifications,
	unused_results,
	clippy::pedantic
)] // from https://github.com/rust-unofficial/patterns/blob/master/anti_patterns/deny-warnings.md

use std::{any, mem, raw};

/// Implemented on all types, it provides helper methods to determine whether a type is `TraitObject`, `Slice` or `Concrete`, and work with them respectively.
pub trait Type {
	/// Enum describing whether a type is `TraitObject`, `Slice` or `Concrete`.
	const METATYPE: MetaType;
	/// Type of metadata for type.
	type Meta: 'static;
	/// Helper method describing whether a type is `TraitObject`, `Slice` or `Concrete`.
	fn meta_type(&self) -> MetaType {
		Self::METATYPE
	}
	/// Retrieve [TraitObject], [Slice] or [Concrete] meta data respectively for a type
	fn meta(&self) -> Self::Meta;
	/// Retrieve pointer to the data
	fn data(&self) -> *const ();
	/// Retrieve mut pointer to the data
	fn data_mut(&mut self) -> *mut ();
	/// Create a `Box<Self>` with the provided `Self::Meta` but with the allocated data uninitialized.
	unsafe fn uninitialized_box(t: Self::Meta) -> Box<Self>;
}
/// Meta type of a type
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum MetaType {
	/// Trait object, thus unsized
	TraitObject,
	/// Slice, thus unsized
	Slice,
	/// Sized type
	Concrete,
}

/// Meta data for a trait object
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct TraitObject {
	/// Address of vtable
	pub vtable: &'static (),
}
/// Meta data for a slice
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Slice {
	/// Number of elements in the slice
	pub len: usize,
}
/// Meta data for a concrete, sized type
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Concrete;

impl<T: ?Sized> Type for T {
	#[doc(hidden)]
	default const METATYPE: MetaType = MetaType::TraitObject;
	#[doc(hidden)]
	default type Meta = TraitObject;
	#[inline]
	default fn meta(&self) -> Self::Meta {
		assert_eq!(
			(mem::size_of::<&Self>(), mem::align_of::<&Self>()),
			(
				mem::size_of::<raw::TraitObject>(),
				mem::align_of::<raw::TraitObject>()
			)
		);
		let trait_object: raw::TraitObject = unsafe { mem::transmute_copy(&self) };
		assert_eq!(
			trait_object.data as *const (),
			self as *const Self as *const ()
		);
		let ret = TraitObject {
			vtable: unsafe { &*trait_object.vtable },
		};
		assert_eq!(
			any::TypeId::of::<Self::Meta>(),
			any::TypeId::of::<TraitObject>()
		);
		unsafe { mem::transmute_copy(&ret) }
	}
	#[inline]
	default fn data(&self) -> *const () {
		assert_eq!(
			(mem::size_of::<&Self>(), mem::align_of::<&Self>()),
			(
				mem::size_of::<raw::TraitObject>(),
				mem::align_of::<raw::TraitObject>()
			)
		);
		let trait_object: raw::TraitObject = unsafe { mem::transmute_copy(&self) };
		assert_eq!(
			trait_object.data as *const (),
			self as *const Self as *const ()
		);
		self as *const Self as *const ()
	}
	#[inline]
	default fn data_mut(&mut self) -> *mut () {
		assert_eq!(
			(mem::size_of::<&Self>(), mem::align_of::<&Self>()),
			(
				mem::size_of::<raw::TraitObject>(),
				mem::align_of::<raw::TraitObject>()
			)
		);
		let trait_object: raw::TraitObject = unsafe { mem::transmute_copy(&self) };
		assert_eq!(trait_object.data, self as *mut Self as *mut ());
		self as *mut Self as *mut ()
	}
	default unsafe fn uninitialized_box(t: Self::Meta) -> Box<Self> {
		assert_eq!(
			any::TypeId::of::<Self::Meta>(),
			any::TypeId::of::<TraitObject>()
		);
		let t: TraitObject = mem::transmute_copy(&t);
		assert_eq!(
			(mem::size_of::<&Self>(), mem::align_of::<&Self>()),
			(
				mem::size_of::<raw::TraitObject>(),
				mem::align_of::<raw::TraitObject>()
			)
		);
		let object: &Self = mem::transmute_copy(&raw::TraitObject {
			data: &mut (),
			vtable: t.vtable as *const () as *mut (),
		}); // ptr::null_mut() causes llvm to assume below is unreachable
		let (size, align) = (mem::size_of_val(object), mem::align_of_val(object));
		let mut backing = Vec::with_capacity(size);
		backing.set_len(size);
		let backing: Box<[u8]> = backing.into_boxed_slice();
		assert_eq!(backing.get_unchecked(0) as *const u8 as usize % align, 0);
		let backing = mem::transmute::<_, raw::TraitObject>(backing); // TODO: work out how to make backing sufficiently aligned
		assert_eq!(
			(mem::size_of::<Box<Self>>(), mem::align_of::<Box<Self>>()),
			(
				mem::size_of::<raw::TraitObject>(),
				mem::align_of::<raw::TraitObject>()
			)
		);
		mem::transmute_copy(&raw::TraitObject {
			data: backing.data,
			vtable: t.vtable as *const () as *mut (),
		})
	}
}
#[doc(hidden)]
impl<T: Sized> Type for T {
	const METATYPE: MetaType = MetaType::Concrete;
	type Meta = Concrete;
	#[inline]
	fn meta(&self) -> Self::Meta {
		Concrete
	}
	#[inline]
	fn data(&self) -> *const () {
		self as *const Self as *const ()
	}
	#[inline]
	fn data_mut(&mut self) -> *mut () {
		self as *mut Self as *mut ()
	}
	unsafe fn uninitialized_box(_: Self::Meta) -> Box<Self> {
		box mem::uninitialized()
	}
}
#[doc(hidden)]
impl<T: Sized> Type for [T] {
	const METATYPE: MetaType = MetaType::Slice;
	type Meta = Slice;
	#[inline]
	fn meta(&self) -> Self::Meta {
		assert_eq!(
			(mem::size_of_val(self), mem::align_of_val(self)),
			(mem::size_of::<T>() * self.len(), mem::align_of::<T>())
		);
		Slice { len: self.len() }
	}
	#[inline]
	fn data(&self) -> *const () {
		self.as_ptr() as *const ()
	}
	#[inline]
	fn data_mut(&mut self) -> *mut () {
		self.as_mut_ptr() as *mut ()
	}
	unsafe fn uninitialized_box(t: Self::Meta) -> Box<Self> {
		let mut backing = Vec::<T>::with_capacity(t.len);
		backing.set_len(t.len);
		backing.into_boxed_slice()
	}
}
#[doc(hidden)]
impl Type for str {
	const METATYPE: MetaType = MetaType::Slice;
	type Meta = Slice;
	#[inline]
	fn meta(&self) -> Self::Meta {
		assert_eq!(
			(mem::size_of_val(self), mem::align_of_val(self)),
			(self.len(), 1)
		);
		Slice { len: self.len() }
	}
	#[inline]
	fn data(&self) -> *const () {
		self.as_ptr() as *const ()
	}
	#[inline]
	fn data_mut(&mut self) -> *mut () {
		unsafe { self.as_bytes_mut() }.as_mut_ptr() as *mut ()
	}
	unsafe fn uninitialized_box(t: Self::Meta) -> Box<Self> {
		let mut backing = Vec::<u8>::with_capacity(t.len);
		backing.set_len(t.len);
		String::from_utf8_unchecked(backing).into_boxed_str()
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::cast_ptr_alignment, clippy::shadow_unrelated)]
	use super::{MetaType, Type};
	use std::{any, mem, ptr};

	#[test]
	fn abc() {
		let a: Box<usize> = Box::new(123);
		assert_eq!(Type::meta_type(&*a), MetaType::Concrete);
		assert_eq!(Type::meta_type(&a), MetaType::Concrete);
		let a: Box<dyn any::Any> = a;
		assert_eq!(Type::meta_type(&*a), MetaType::TraitObject);
		assert_eq!(Type::meta_type(&a), MetaType::Concrete);
		let meta = Type::meta(&*a); // : TraitObject
		let mut b: Box<dyn any::Any> = unsafe { Type::uninitialized_box(meta) };
		assert_eq!(mem::size_of_val(&*b), mem::size_of::<usize>());
		unsafe { ptr::write(&mut *b as *mut dyn any::Any as *mut usize, 456_usize) };
		let x: usize = *Box::<dyn any::Any>::downcast(b).unwrap();
		assert_eq!(x, 456);
		let a: &[usize] = &[1, 2, 3];
		assert_eq!(Type::meta_type(a), MetaType::Slice);
		let a: Box<[usize]> = vec![1_usize, 2, 3].into_boxed_slice();
		assert_eq!(Type::meta_type(&*a), MetaType::Slice);
		assert_eq!(Type::meta_type(&a), MetaType::Concrete);
		let a: &str = "abc";
		assert_eq!(Type::meta_type(a), MetaType::Slice);
		assert_eq!(Type::meta_type(&a), MetaType::Concrete);
	}
}
