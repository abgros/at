#![no_std]
#![warn(clippy::pedantic)]
#![deny(missing_docs)]
#![allow(clippy::inline_always)]

//! Various helpers for indexing slices.
//! This crate provides three methods for indexing slices: `at`, `ref_at`, and `mut_at`.
//! These methods offer a few benefits over standard indexing:
//! - They work for any integer type, rather than just `usize`
//! - They support Pythonesque negative indices; for example, `nums.at(-1)` returns the last element
//! - You explicitly specify whether you're indexing by value (for Copy types), reference, or mutable reference,
//!   rather than the compiler "magically" choosing the right kind of access
//! - You can disable *all* bounds checks across the entire program by activating the `unsafe-unchecked` feature;
//!   this is not recommended unless you absolutely need the performance gains
//!
//! All this happens with zero runtime overhead compared to standard indexing.
//! However, note that checking the validity of signed types is slightly more complex
//! than for a `usize` due to negative indexing. Signed indexing does not incur any
//! overhead when the index is known at compile time.
//!
//! # Examples
//! ```
//! use at::At;
//!
//! let mut v = vec![8, 2, 1, 0];
//! assert_eq!(v.at(-1), 0);
//! assert_eq!(v.ref_at(2), &1);
//! assert_eq!(v.mut_at(-3), &mut 2);
//! ```

#[cfg(feature = "unsafe-unchecked")]
use core::hint::unreachable_unchecked;

mod private {
	pub trait ToIndex: TryInto<isize> + TryInto<usize> + core::fmt::Debug + Copy {}
	impl<T: TryInto<isize> + TryInto<usize> + core::fmt::Debug + Copy> ToIndex for T {}
}

// Trait alias for TryInto<isize> + TryInto<usize> + core::fmt::Debug + Copy
use private::ToIndex;

#[inline(always)]
fn check_index(idx: impl ToIndex, len: usize) -> Option<usize> {
	let resolved = if let Ok(unsigned_index) = idx.try_into() {
		unsigned_index
	} else {
		let signed_index = idx.try_into().ok()?;
		// If this overflows, the index is guaranteed invalid (this is handled at the end of this function).
		// Proof: `signed_index` must be negative; otherwise, the previous branch would have succeeded.
		// Thus `signed_index` is any negative number in `isize::MIN..0`. After the addition,
		// `resolved` is in `len + isize::MIN..len`. If the length is extremely large, that is `len > isize::MAX`,
		// (only possible for ZST slices) overflow does not occur. Otherwise, the wrapped range
		// is `len + isize::MIN..0` which becomes `len + isize::MAX + 1..=usize::MAX`. But since we
		// know that `len` is at most `isize::MAX` in this case, the wrapped range is always invalid.
		// Therefore, we use the wrapping method to discourage the compiler from adding pointless runtime checks.
		len.wrapping_add_signed(signed_index)
	};

	(resolved < len).then_some(resolved)
}

#[cfg(not(feature = "unsafe-unchecked"))]
#[inline(never)]
fn panic_bounds_check(idx: impl ToIndex, len: usize) -> ! {
	panic!("index out of bounds: the len is {len} but the index is {idx:?}")
}

/// This trait provides the `at`, `ref_at`, and `mut_at` methods for slices
/// as well as any type that can be deferenced to a slice.
pub trait At {
	/// Access a particular index of a `Copy` type. Panics if the index is out of bounds.
	///
	/// # Examples
	/// ```
	/// use at::At;
	/// let a = [1, 2, 3];
	///
	/// assert_eq!(a.at(2), 3);
	/// assert_eq!(a.at(-2), 2);
	/// ```
	#[inline(always)]
	fn at<T>(&self, idx: impl ToIndex) -> T
	where
		Self: AsRef<[T]>,
		T: Copy,
	{
		let slice = self.as_ref();
		let len = slice.len();

		match check_index(idx, len) {
			Some(i) => slice[i],
			#[cfg(feature = "unsafe-unchecked")]
			None => unsafe { unreachable_unchecked() },
			#[cfg(not(feature = "unsafe-unchecked"))]
			None => panic_bounds_check(idx, len),
		}
	}

	/// Access a particular index by reference. Panics if the index is out of bounds.
	///
	/// # Examples
	/// ```
	/// use at::At;
	/// let a = [1, 2, 3];
	///
	/// assert_eq!(a.ref_at(2), &3);
	/// assert_eq!(a.ref_at(-2), &2);
	/// ```
	#[inline(always)]
	fn ref_at<T>(&self, idx: impl ToIndex) -> &T
	where
		Self: AsRef<[T]>,
	{
		let slice = self.as_ref();
		let len = slice.len();

		match check_index(idx, len) {
			Some(i) => &slice[i],
			#[cfg(feature = "unsafe-unchecked")]
			None => unsafe { unreachable_unchecked() },
			#[cfg(not(feature = "unsafe-unchecked"))]
			None => panic_bounds_check(idx, len),
		}
	}

	/// Access a particular index by mutable reference. Panics if the index is out of bounds.
	///
	/// # Examples
	/// ```
	/// use at::At;
	/// let mut a = [1, 2, 3];
	///
	/// assert_eq!(a.mut_at(2), &mut 3);
	/// assert_eq!(a.mut_at(-2), &mut 2);
	/// ```
	#[inline(always)]
	fn mut_at<T>(&mut self, idx: impl ToIndex) -> &mut T
	where
		Self: AsMut<[T]>,
	{
		let slice = self.as_mut();
		let len = slice.len();

		match check_index(idx, len) {
			Some(i) => &mut slice[i],
			#[cfg(feature = "unsafe-unchecked")]
			None => unsafe { unreachable_unchecked() },
			#[cfg(not(feature = "unsafe-unchecked"))]
			None => panic_bounds_check(idx, len),
		}
	}
}

impl<T> At for T {}

mod test {
	#[cfg(test)]
	use crate::At;

	#[test]
	fn test_positive() {
		extern crate std;
		use std::vec;

		let mut v = vec![1, 2, 3];
		assert_eq!(v.at(0u8), 1);
		assert_eq!(v.ref_at(1i128), &2);
		assert_eq!(v.mut_at(2isize), &mut 3);
	}

	#[test]
	fn test_negative() {
		let mut v = [4, 5, 6];
		assert_eq!(v.at(-1i8), 6);
		assert_eq!(v.ref_at(-2i128), &5);
		assert_eq!(v.mut_at(-3isize), &mut 4);
	}

	#[test]
	#[should_panic(expected = "index out of bounds: the len is 1 but the index is -2")]
	fn test_panic() {
		let s = ["hi"];
		let _ = s.at(-2);
	}

	#[test]
	fn test_zst() {
		let giant = [(); usize::MAX];
		giant.at(-1);
		giant.at(usize::MAX - 1);
		giant.at(isize::MIN);
	}
}
