//! VBox is a type erased Box of trait object that stores the vtable pointer.
//!
//! `VBox` is just like a `Box<dyn Trait>` but erases type `Trait` so that to
//! use it, there is no need to have `Trait` as one of its type parameters.
//! Only the creator and the consumer needs to agree on the type
//! parameters.
//!
//! Internally, it stores the trait object's data pointer in a `Box<dyn Any +
//! Send>`, so that the `Drop::drop()` will be called when the wrapper is
//! dropped. And it stores the vtable pointer in another `usize` to make sure it
//! is `Send`.
//!
//! # Example
//! ```
//! # use std::fmt::{Debug, Display};
//! # use vbox::{from_vbox, into_vbox, VBox};
//! // Pack a u64 into a `Debug` trait object and erase the type `Debug`.
//! let vbox: VBox = into_vbox!(dyn Debug, 10u64);
//!
//! // Unpack to different trait object will panic:
//! // let _panic = from_vbox!(dyn Display, vbox);
//!
//! // Unpack the `Debug` trait object.
//! let unpacked: Box<dyn Debug> = from_vbox!(dyn Debug, vbox);
//!
//! assert_eq!("10", format!("{:?}", unpacked));
//! ```

use std::any::Any;
use std::any::TypeId;

/// A type erased Box of trait object that stores the vtable pointer.
///
/// This is just like a `Box<dyn Trait>` but erases type `Trait` so that the
/// channel for sending it does not need to have `Trait` as one of its type
/// parameters. Only the sending end and the receiving end need to agree on the
/// type parameters.
///
/// Internally, it stores the trait object's data pointer in a `Box<dyn Any>`,
/// so that the `Drop::drop()` will be called when the wrapper is dropped.
/// And it stores the vtable pointer in another `usize` to make sure it is
/// `Send`.
pub struct VBox {
    /// The data pointer.
    ///
    /// Wrap it in a `Box` to make sure it is dropped when `VBox` is dropped.
    data: Box<dyn Any + Send>,

    /// The vtable pointer.
    ///
    /// Stored in `usize` to make sure it is `Send`.
    vtable: usize,

    /// Type id of `&dyn Trait`, for debugging.
    type_id: TypeId,
}

impl VBox {
    /// Create a new VBox. Do not use it directly. Use [`into_vbox!`] instead.
    pub fn new(
        data: Box<dyn Any + Send>,
        vtable: usize,
        type_id: TypeId,
    ) -> Self {
        VBox {
            data,
            vtable,
            type_id,
        }
    }

    /// Unpack the `VBox` and return the fields to rebuild the original trait
    /// object. Do not use it directly. Use [`from_vbox!`] instead.
    pub fn unpack(self) -> (Box<dyn Any + Send>, usize, TypeId) {
        (self.data, self.vtable, self.type_id)
    }
}

/// Create a [`VBox`] from a user defined type `T`.
///
/// The built `VBox` is another form of `Box<dyn Trait>`, where `T: Trait`.
///
/// See: [crate doc](crate)
#[macro_export]
macro_rules! into_vbox {
    ($t: ty, $v: expr) => {{
        let type_id = {
            let trait_obj_ref: &$t = &$v;
            ::std::any::Any::type_id(trait_obj_ref)
        };

        let vtable = {
            let fat_ptr: *const $t = &$v;
            let (_data, vtable): (*const (), *const ()) =
                unsafe { ::std::mem::transmute(fat_ptr) };
            vtable as usize
        };

        VBox::new(Box::new($v), vtable, type_id)
    }};
}

/// Consume [`VBox`] and reconstruct the original trait object: `Box<dyn
/// Trait>`.
///
/// It retrieve data pointer from `VBox.data` and the vtable pointer from
/// `VBox.vtable`. Then it puts them together to reconstruct the fat pointer for
/// the trait object.
///
/// See: [crate doc](crate)
#[macro_export]
macro_rules! from_vbox {
    ($t: ty, $v: expr) => {{
        let (data, vtable, type_id) = $v.unpack();

        let any_fat_ptr: *const dyn ::core::any::Any = Box::into_raw(data);
        let (data_ptr, _vtable): (*const (), *const ()) =
            unsafe { ::std::mem::transmute(any_fat_ptr) };

        let vtable_ptr = vtable as *const ();

        let fat_ptr: *mut $t =
            unsafe { ::std::mem::transmute((data_ptr, vtable_ptr)) };

        let ret = unsafe { Box::from_raw(fat_ptr) };

        {
            let trait_obj_ref = &*ret;
            debug_assert_eq!(
                ::std::any::Any::type_id(trait_obj_ref),
                type_id,
                "expected type_id: {:?}, actual type_id: {:?}",
                ::std::any::Any::type_id(trait_obj_ref),
                type_id
            );
        }

        ret
    }};
}
