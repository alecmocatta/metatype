//! Helper methods to determine whether a type is `TraitObject`, `Slice` or
//! `Concrete`, and work with them respectively.
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
//! assert_eq!(Type::meta_type(&*a), MetaType::Concrete);
//! let a: Box<dyn any::Any> = a;
//! assert_eq!(Type::meta_type(&*a), MetaType::TraitObject);
//!
//! let a = [123,456];
//! assert_eq!(Type::meta_type(&a), MetaType::Concrete);
//! let a: &[i32] = &a;
//! assert_eq!(Type::meta_type(a), MetaType::Slice);
//!
//! let a: Box<dyn any::Any> = Box::new(123);
//! let meta: TraitObject = type_coerce(Type::meta(&*a));
//! println!("vtable: {:?}", meta.vtable);
//! ```
//!
//! # Note
//!
//! This currently requires Rust nightly for the `ptr_metadata`, `specialization`
//! and `arbitrary_self_types_pointers` features.

#![doc(html_root_url = "https://docs.rs/metatype/0.2.1")]
#![feature(arbitrary_self_types_pointers)]
#![feature(ptr_metadata)]
#![feature(specialization)]
#![warn(
	missing_copy_implementations,
	missing_debug_implementations,
	missing_docs,
	trivial_casts,
	trivial_numeric_casts,
	unused_import_braces,
	unused_qualifications,
	unused_results,
	clippy::pedantic
)] // from https://github.com/rust-unofficial/patterns/blob/master/anti_patterns/deny-warnings.md
#![allow(
	clippy::must_use_candidate,
	clippy::not_unsafe_ptr_arg_deref,
	clippy::use_self,
	clippy::missing_panics_doc,
	incomplete_features
)]

use std::{
	any::{type_name, TypeId}, hash::{Hash, Hasher}, marker::PhantomData, mem::{align_of, align_of_val, forget, size_of, size_of_val, transmute_copy}, ptr::{slice_from_raw_parts_mut, NonNull}
};

/// Implemented on all types, it provides helper methods to determine whether a type is `TraitObject`, `Slice` or `Concrete`, and work with them respectively.
pub trait Type {
	/// Enum describing whether a type is `TraitObject`, `Slice` or `Concrete`.
	const METATYPE: MetaType;
	/// Type of metadata for type.
	type Meta: 'static;
	/// Helper method describing whether a type is `TraitObject`, `Slice` or `Concrete`.
	fn meta_type(self: *const Self) -> MetaType {
		Self::METATYPE
	}
	/// Retrieve [`TraitObject`], [`Slice`] or [`Concrete`] meta data respectively for a type
	fn meta(self: *const Self) -> Self::Meta;
	/// Retrieve pointer to the data
	fn data(self: *const Self) -> *const ();
	/// Retrieve mut pointer to the data
	fn data_mut(self: *mut Self) -> *mut ();
	/// Create a dangling non-null `*const Self` with the provided `Self::Meta`.
	fn dangling(t: Self::Meta) -> NonNull<Self>;
	/// Create a `*mut Self` with the provided `Self::Meta`.
	fn fatten(thin: *mut (), t: Self::Meta) -> *mut Self;
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
	default fn meta(self: *const Self) -> Self::Meta {
		let ret = TraitObject {
			vtable: unsafe { transmute_coerce(std::ptr::metadata(self)) },
		};
		type_coerce(ret)
	}
	#[inline]
	default fn data(self: *const Self) -> *const () {
		self.cast()
	}
	#[inline]
	default fn data_mut(self: *mut Self) -> *mut () {
		self.cast()
	}
	#[inline]
	default fn dangling(t: Self::Meta) -> NonNull<Self> {
		let t: TraitObject = type_coerce(t);
		// align_of_val requires a reference: https://github.com/rust-lang/rfcs/issues/2017
		// so to placate miri let's create one that's plausibly valid
		let fake_thin = {
			#[allow(dead_code)]
			#[repr(align(64))]
			struct Backing(u8);
			static BACKING: Backing = Backing(0);
			let backing: *const _ = &BACKING;
			backing.cast::<()>().cast_mut()
		};
		let dangling_unaligned: NonNull<Self> =
			NonNull::new(Self::fatten(fake_thin, type_coerce(t))).unwrap();
		let dangling_unaligned: &Self = unsafe { dangling_unaligned.as_ref() };
		let align = align_of_val(dangling_unaligned);
		NonNull::new(Self::fatten(align as _, type_coerce(t))).unwrap()
	}
	#[inline]
	default fn fatten(thin: *mut (), t: Self::Meta) -> *mut Self {
		let t: TraitObject = type_coerce(t);
		let vtable: *const () = t.vtable;
		let vtable = vtable.cast_mut();
		std::ptr::from_raw_parts_mut(thin, unsafe { transmute_coerce(vtable) })
	}
}
#[doc(hidden)]
impl<T: Sized> Type for T {
	const METATYPE: MetaType = MetaType::Concrete;
	type Meta = Concrete;
	#[inline]
	fn meta(self: *const Self) -> Self::Meta {
		Concrete
	}
	#[inline]
	fn data(self: *const Self) -> *const () {
		self.cast()
	}
	#[inline]
	fn data_mut(self: *mut Self) -> *mut () {
		self.cast()
	}
	fn dangling(_t: Self::Meta) -> NonNull<Self> {
		NonNull::dangling()
	}
	fn fatten(thin: *mut (), _t: Self::Meta) -> *mut Self {
		thin.cast()
	}
}
#[doc(hidden)]
impl<T: Sized> Type for [T] {
	const METATYPE: MetaType = MetaType::Slice;
	type Meta = Slice;
	#[allow(clippy::manual_slice_size_calculation)]
	#[inline]
	fn meta(self: *const Self) -> Self::Meta {
		let self_ = unsafe { &*self }; // https://github.com/rust-lang/rfcs/issues/2017
		assert_eq!(
			(size_of_val(self_), align_of_val(self_)),
			(size_of::<T>() * self_.len(), align_of::<T>())
		);
		Slice { len: self_.len() }
	}
	#[inline]
	fn data(self: *const Self) -> *const () {
		self.cast()
	}
	#[inline]
	fn data_mut(self: *mut Self) -> *mut () {
		self.cast()
	}
	fn dangling(t: Self::Meta) -> NonNull<Self> {
		let slice = slice_from_raw_parts_mut(NonNull::<T>::dangling().as_ptr(), t.len);
		unsafe { NonNull::new_unchecked(slice) }
	}
	fn fatten(thin: *mut (), t: Self::Meta) -> *mut Self {
		slice_from_raw_parts_mut(thin.cast(), t.len)
	}
}
#[doc(hidden)]
impl Type for str {
	const METATYPE: MetaType = MetaType::Slice;
	type Meta = Slice;
	#[inline]
	fn meta(self: *const Self) -> Self::Meta {
		let self_ = unsafe { &*self }; // https://github.com/rust-lang/rfcs/issues/2017
		assert_eq!((size_of_val(self_), align_of_val(self_)), (self_.len(), 1));
		Slice { len: self_.len() }
	}
	#[inline]
	fn data(self: *const Self) -> *const () {
		self.cast()
	}
	#[inline]
	fn data_mut(self: *mut Self) -> *mut () {
		self.cast()
	}
	fn dangling(t: Self::Meta) -> NonNull<Self> {
		let bytes: *mut [u8] = <[u8]>::dangling(t).as_ptr();
		unsafe { NonNull::new_unchecked(bytes as *mut Self) }
	}
	fn fatten(thin: *mut (), t: Self::Meta) -> *mut Self {
		<[u8]>::fatten(thin, t) as *mut Self
	}
}

unsafe fn transmute_coerce<A, B>(a: A) -> B {
	assert_eq!(
		(size_of::<A>(), align_of::<A>()),
		(size_of::<B>(), align_of::<B>()),
		"can't transmute_coerce {} to {} as sizes/alignments differ",
		type_name::<A>(),
		type_name::<B>()
	);
	let b = transmute_copy(&a);
	forget(a);
	b
}

/// Convert from one type parameter to another, where they are the same type.
/// Panics with an explanatory message if the types differ.
///
/// In almost all circumstances this isn't needed, but it can be very useful in
/// cases like [rust-lang/rust#50318](https://github.com/rust-lang/rust/issues/50318).
pub fn type_coerce<A, B>(a: A) -> B {
	try_type_coerce(a)
		.unwrap_or_else(|| panic!("can't coerce {} to {}", type_name::<A>(), type_name::<B>()))
}

/// Convert from one type parameter to another, where they are the same type.
/// Returns `None` if the types differ.
///
/// In almost all circumstances this isn't needed, but it can be very useful in
/// cases like [rust-lang/rust#50318](https://github.com/rust-lang/rust/issues/50318).
pub fn try_type_coerce<A, B>(a: A) -> Option<B> {
	trait Eq<B> {
		fn eq(self) -> Option<B>;
	}

	struct Foo<A, B>(A, PhantomData<fn(B)>);

	impl<A, B> Eq<B> for Foo<A, B> {
		default fn eq(self) -> Option<B> {
			None
		}
	}
	#[allow(clippy::mismatching_type_param_order)]
	impl<A> Eq<A> for Foo<A, A> {
		fn eq(self) -> Option<A> {
			Some(self.0)
		}
	}

	Foo::<A, B>(a, PhantomData).eq()
}

/// Gets an identifier which is globally unique to the specified type. This
/// function will return the same value for a type regardless of whichever crate
/// it is invoked in.
pub fn type_id<T: ?Sized + 'static>() -> u64 {
	let type_id = TypeId::of::<T>();
	let mut hasher = std::collections::hash_map::DefaultHasher::new();
	type_id.hash(&mut hasher);
	hasher.finish()
}

#[cfg(test)]
mod tests {
	#![allow(clippy::cast_ptr_alignment, clippy::shadow_unrelated)]
	use super::{type_coerce, MetaType, Slice, TraitObject, Type};
	use std::{any, ptr::NonNull};

	#[test]
	fn abc() {
		let a: Box<usize> = Box::new(123);
		assert_eq!(Type::meta_type(&*a), MetaType::Concrete);
		assert_eq!(Type::meta_type(&a), MetaType::Concrete);
		let a: Box<dyn any::Any> = a;
		assert_eq!(Type::meta_type(&*a), MetaType::TraitObject);
		assert_eq!(Type::meta_type(&a), MetaType::Concrete);
		let meta: TraitObject = type_coerce(Type::meta(&*a));
		let dangling = <dyn any::Any as Type>::dangling(type_coerce(meta));
		let _fat = <dyn any::Any as Type>::fatten(dangling.as_ptr().cast(), type_coerce(meta));
		let mut x: usize = 0;
		let x_ptr: *mut usize = &mut x;
		let mut x_ptr: NonNull<dyn any::Any> = NonNull::new(<dyn any::Any as Type>::fatten(
			x_ptr.cast(),
			type_coerce(meta),
		))
		.unwrap();
		let x_ref: &mut dyn any::Any = unsafe { x_ptr.as_mut() };
		let x_ref: &mut usize = x_ref.downcast_mut().unwrap();
		*x_ref = 123;
		assert_eq!(x, 123);

		let a: &[usize] = &[1, 2, 3];
		assert_eq!(Type::meta_type(a), MetaType::Slice);
		let dangling = <[String] as Type>::dangling(Slice { len: 100 });
		let _fat = <[String] as Type>::fatten(dangling.as_ptr().cast(), Slice { len: 100 });

		let a: Box<[usize]> = vec![1_usize, 2, 3].into_boxed_slice();
		assert_eq!(Type::meta_type(&*a), MetaType::Slice);
		assert_eq!(Type::meta_type(&a), MetaType::Concrete);

		let a: &str = "abc";
		assert_eq!(Type::meta_type(a), MetaType::Slice);
		assert_eq!(Type::meta_type(&a), MetaType::Concrete);
		let dangling = <str as Type>::dangling(Slice { len: 100 });
		let _fat = <str as Type>::fatten(dangling.as_ptr().cast(), Slice { len: 100 });
	}
}
