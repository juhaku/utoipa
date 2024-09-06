use crate::component::{GenericType, TypeTree};
use crate::schema_type::SchemaType;

use super::validation::NumberValue;

pub trait Validator {
    fn is_valid(&self) -> Result<(), &'static str>;
}

pub struct IsNumber<'a>(pub &'a SchemaType<'a>);

impl Validator for IsNumber<'_> {
    fn is_valid(&self) -> Result<(), &'static str> {
        if self.0.is_number() {
            Ok(())
        } else {
            Err("can only be used with `number` type")
        }
    }
}

pub struct IsString<'a>(pub(super) &'a SchemaType<'a>);

impl Validator for IsString<'_> {
    fn is_valid(&self) -> Result<(), &'static str> {
        if self.0.is_string() {
            Ok(())
        } else {
            Err("can only be used with `string` type")
        }
    }
}

pub struct IsInteger<'a>(&'a SchemaType<'a>);

impl Validator for IsInteger<'_> {
    fn is_valid(&self) -> Result<(), &'static str> {
        if self.0.is_integer() {
            Ok(())
        } else {
            Err("can only be used with `integer` type")
        }
    }
}

pub struct IsVec<'a>(pub(super) &'a TypeTree<'a>);

impl Validator for IsVec<'_> {
    fn is_valid(&self) -> Result<(), &'static str> {
        if self.0.generic_type == Some(GenericType::Vec) {
            Ok(())
        } else {
            Err("can only be used with `Vec`, `array` or `slice` types")
        }
    }
}

pub struct AboveZeroUsize<'a>(pub(super) &'a NumberValue);

impl Validator for AboveZeroUsize<'_> {
    fn is_valid(&self) -> Result<(), &'static str> {
        let usize: usize = self
            .0
            .try_from_str()
            .map_err(|_| "invalid type, expected `usize`")?;

        if usize != 0 {
            Ok(())
        } else {
            Err("can only be above zero value")
        }
    }
}

pub struct AboveZeroF64<'a>(pub(super) &'a NumberValue);

impl Validator for AboveZeroF64<'_> {
    fn is_valid(&self) -> Result<(), &'static str> {
        let float: f64 = self
            .0
            .try_from_str()
            .map_err(|_| "invalid type, expected `f64`")?;
        if float > 0.0 {
            Ok(())
        } else {
            Err("can only be above zero value")
        }
    }
}

pub struct ValidatorChain<'c> {
    inner: &'c dyn Validator,
    next: Option<&'c dyn Validator>,
}

impl Validator for ValidatorChain<'_> {
    fn is_valid(&self) -> Result<(), &'static str> {
        self.inner.is_valid().and_then(|_| {
            if let Some(validator) = self.next.as_ref() {
                validator.is_valid()
            } else {
                // if there is no next validator consider it valid
                Ok(())
            }
        })
    }
}

impl<'c> ValidatorChain<'c> {
    pub fn new(validator: &'c dyn Validator) -> Self {
        Self {
            inner: validator,
            next: None,
        }
    }

    pub fn next(mut self, validator: &'c dyn Validator) -> Self {
        self.next = Some(validator);

        self
    }
}
