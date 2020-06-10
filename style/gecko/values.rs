/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

//! Different kind of helpers to interact with Gecko values.

use crate::counter_style::{Symbol, Symbols};
use crate::gecko_bindings::bindings;
use crate::gecko_bindings::structs::CounterStylePtr;
use crate::values::generics::CounterStyle;
use crate::values::Either;
use crate::Atom;
use app_units::Au;
use cssparser::RGBA;
use std::cmp::max;

/// Convert a given RGBA value to `nscolor`.
pub fn convert_rgba_to_nscolor(rgba: &RGBA) -> u32 {
    ((rgba.alpha as u32) << 24) |
        ((rgba.blue as u32) << 16) |
        ((rgba.green as u32) << 8) |
        (rgba.red as u32)
}

/// Convert a given `nscolor` to a Servo RGBA value.
pub fn convert_nscolor_to_rgba(color: u32) -> RGBA {
    RGBA::new(
        (color & 0xff) as u8,
        (color >> 8 & 0xff) as u8,
        (color >> 16 & 0xff) as u8,
        (color >> 24 & 0xff) as u8,
    )
}

/// Ensures that `width`, if non-zero, is at least one device pixel. These
/// widths get floored to device pixels anyhow at rendering time.
#[inline]
pub fn round_border_width(width: Au, au_per_device_px: Au) -> Au {
    if width == Au(0) {
        Au(0)
    } else {
        max(au_per_device_px, width)
    }
}

impl CounterStyle {
    /// Convert this counter style to a Gecko CounterStylePtr.
    #[inline]
    pub fn to_gecko_value(&self, gecko_value: &mut CounterStylePtr) {
        unsafe { bindings::Gecko_CounterStyle_ToPtr(self, gecko_value) }
    }

    /// Convert Gecko CounterStylePtr to CounterStyle or String.
    pub fn from_gecko_value(gecko_value: &CounterStylePtr) -> Either<Self, String> {
        use crate::values::CustomIdent;

        let name = unsafe { bindings::Gecko_CounterStyle_GetName(gecko_value) };
        if !name.is_null() {
            let name = unsafe { Atom::from_raw(name) };
            debug_assert_ne!(name, atom!("none"));
            Either::First(CounterStyle::Name(CustomIdent(name)))
        } else {
            let anonymous =
                unsafe { bindings::Gecko_CounterStyle_GetAnonymous(gecko_value).as_ref() }.unwrap();
            let symbols = &anonymous.mSymbols;
            if anonymous.mSingleString {
                debug_assert_eq!(symbols.len(), 1);
                Either::Second(symbols[0].to_string())
            } else {
                let symbol_type = anonymous.mSymbolsType;
                let symbols = symbols
                    .iter()
                    .map(|gecko_symbol| Symbol::String(gecko_symbol.to_string().into()))
                    .collect();
                Either::First(CounterStyle::Symbols(symbol_type, Symbols(symbols)))
            }
        }
    }
}
