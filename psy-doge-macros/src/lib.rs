/*
concat from https://github.com/inspier/array-concat/blob/bc9e8d0f9a2fcf177286369d976ec38a0a874cc2/src/lib.rs
MIT License

Copyright (c) 2021 inspier

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/
/// Computes total size of all provided const arrays.
#[macro_export]
macro_rules! const_concat_arrays_size {
    ($( $array:expr ),*) => {{
        0 $(+ $array.len())*
    }};
}

/// Concatenates provided arrays.
#[macro_export]
macro_rules! const_concat_arrays {
    ($( $array:expr ),*) => ({
        const __ARRAY_SIZE__: usize = $crate::const_concat_arrays_size!($($array),*);

        #[repr(C)]
        struct ArrayConcatDecomposed<T>($([T; $array.len()]),*);

        #[repr(C)]
        union ArrayConcatComposed<T, const N: usize> {
            full: core::mem::ManuallyDrop<[T; N]>,
            decomposed: core::mem::ManuallyDrop<ArrayConcatDecomposed<T>>,
        }

        impl<T, const N: usize> ArrayConcatComposed<T, N> {
            const fn have_same_size(&self) -> bool {
                core::mem::size_of::<[T; N]>() == core::mem::size_of::<Self>()
            }
        }

        let composed = ArrayConcatComposed { decomposed: core::mem::ManuallyDrop::new(ArrayConcatDecomposed ( $($array),* ))};

        // Sanity check that composed's two fields are the same size
        ["Size mismatch"][!composed.have_same_size() as usize];

        // SAFETY: Sizes of both fields in composed are the same so this assignment should be sound
        core::mem::ManuallyDrop::into_inner(unsafe { composed.full })
    });
}