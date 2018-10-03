use std::io::prelude::*;

use diesel::deserialize::{self, FromSql};
use diesel::pg::data_types::PgNumeric;
use diesel::pg::Pg;
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::Numeric;
use num::{BigUint, FromPrimitive, Integer, ToPrimitive, Zero};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Balance(pub u128);

impl<'a> From<&'a Balance> for PgNumeric {
    fn from(balance: &'a Balance) -> Self {
        let integer = balance.to_biguint().unwrap_or_default();
        let mut digits = ToBase10000(Some(integer)).collect::<Vec<_>>();
        digits.reverse();
        let weight = digits.len() as i16;
        PgNumeric::Positive { digits, scale: 0, weight }
    }
}

impl From<Balance> for PgNumeric {
    fn from(bigdecimal: Balance) -> Self {
        (&bigdecimal).into()
    }
}

impl ToSql<Numeric, Pg> for Balance {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        let numeric = PgNumeric::from(self);
        ToSql::<Numeric, Pg>::to_sql(&numeric, out)
    }
}

impl FromSql<Numeric, Pg> for Balance {
    fn from_sql(numeric: Option<&[u8]>) -> deserialize::Result<Self> {
        // FIXME: Use the TryFrom impl when try_from is stable
        let numeric = PgNumeric::from_sql(numeric)?;
        pg_decimal_to_balance(&numeric)
    }
}

fn pg_decimal_to_balance(numeric: &PgNumeric) -> deserialize::Result<Balance> {
    let digits = match *numeric {
        PgNumeric::Positive { ref digits, .. } => digits,
        _ => return Err(Box::from("NaN and Negative are not (yet) supported in Balance")),
    };

    let mut result = BigUint::default();
    for digit in digits {
        result *= BigUint::from(10_000u64);
        result += BigUint::from(*digit as u64);
    }
    Balance::from_biguint(result).ok_or(Box::from("Could not create balance from BigUint"))
}

/// Iterator over the digits of a big uint in base 10k.
/// The digits will be returned in little endian order.
struct ToBase10000(Option<BigUint>);

impl Iterator for ToBase10000 {
    type Item = i16;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.take().map(|v| {
            let (div, rem) = v.div_rem(&BigUint::from(10_000u16));
            if !div.is_zero() {
                self.0 = Some(div);
            }
            rem.to_i16().expect("10000 always fits in an i16")
        })
    }
}

impl Balance {
    #[inline]
    pub fn from_biguint(biguint: BigUint) -> Option<Self> {
        biguint.to_u128().map(Balance)
    }

    #[inline]
    pub fn to_biguint(&self) -> Option<BigUint> {
        BigUint::from_u128(self.0)
    }
}
