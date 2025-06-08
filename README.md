Various utility functions for indexing slices.
This crate provides three methods for indexing slices: `at`, `ref_at`, and `mut_at`.
These methods offer a few benefits over standard indexing:
- They work for any integer type, rather than just `usize`[^0]
- They support Pythonesque negative indices; for example, `nums.at(-1)` returns the last element[^1]
- You explicitly specify whether you're indexing by value (for Copy types), reference, or mutable reference,
  rather than the compiler "magically" choosing the right kind of access
- You can disable *all* bounds checks across the entire program by activating the `unsafe-unchecked` feature;
  this is not recommended unless you absolutely need the performance gains

All this happens with zero runtime overhead compared to standard indexing.
However, note that checking the validity of signed types is slightly more complex
than for a `usize` due to negative indexing. Signed indexing does not incur any
overhead when the index is known at compile time.

# Examples
```rs
use at::{At, RefAt, MutAt};

let mut v = vec![8, 2, 1, 0];
assert_eq!(v.at(-1), 0);
assert_eq!(v.ref_at(2), &1);
assert_eq!(v.mut_at(-3), &mut 2);
```

[^0]: Specifically, the trait bound is `TryInto<isize> + TryInto<usize> + Debug + Copy`.
[^1]: Negative indices are converted into an `isize`, so they cannot be smaller than `isize::MIN`.
     Therefore, `[(); usize::MAX].at(-(usize::MAX as i128))` will panic, even though you might
     expect it to successfully return the first element of the slice.