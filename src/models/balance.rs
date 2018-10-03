use std::error::Error as StdError;
use std::io::prelude::*;

use diesel::deserialize::{self, FromSql};
use diesel::pg::data_types::PgNumeric;
use diesel::pg::Pg;
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::Numeric;

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct Balance(u128);

impl<'a> From<&'a Balance> for PgNumeric {
    fn from(balance: &'a Balance) -> Self {
        u128_to_pg_decimal(balance.0)
    }
}

impl From<Balance> for PgNumeric {
    fn from(balance: Balance) -> Self {
        (&balance).into()
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
        let numeric = PgNumeric::from_sql(numeric)?;
        pg_decimal_to_u128(&numeric).map(Balance)
    }
}

/// Iterator over the digits of a big uint in base 10k.
/// The digits will be returned in little endian order.
struct ToBase10000(Option<u128>);

impl Iterator for ToBase10000 {
    type Item = i16;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.take().map(|v| {
            let rem = v % 10_000u128;
            let div = v / 10_000u128;
            if div != 0 {
                self.0 = Some(div);
            }
            rem as i16
        })
    }
}

fn pg_decimal_to_u128(numeric: &PgNumeric) -> deserialize::Result<u128> {
    let (weight, scale, digits) = match *numeric {
        PgNumeric::Positive { weight, scale, ref digits } => (weight, scale, digits),
        PgNumeric::Negative { .. } => return Err(Box::from(format!("Negative is not supported in u128: {:#?}", numeric))),
        PgNumeric::NaN => return Err(Box::from(format!("NaN is not supported in u128: {:#?}", numeric))),
    };

    if scale != 0 {
        return Err(Box::from(format!("Nonzero scale is not supported in u128: {:#?}", numeric)));
    }

    if (digits.len() as i16) != weight {
        return Err(Box::from(format!("Unexpected weight in Pgnumeric: {:#?}", numeric)));
    }

    let mut result = 0u128;
    for digit in digits {
        result = result
            .checked_mul(10_000u128)
            .and_then(|res| res.checked_add(*digit as u128))
            .ok_or(Box::from("Negative is not supported in u128") as Box<StdError + Send + Sync>)?;
    }
    Ok(result)
}

fn u128_to_pg_decimal(value: u128) -> PgNumeric {
    let mut digits = ToBase10000(Some(value)).collect::<Vec<_>>();
    digits.reverse();
    let weight = digits.len() as i16 - 1;
    PgNumeric::Positive { digits, scale: 0, weight }
}
