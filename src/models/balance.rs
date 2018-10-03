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

// to check binary posgres numeric representation
// psql -U postgres -d challenge -c 'COPY ( SELECT 1000000001000000000000000000 ) TO STDOUT WITH ( FORMAT BINARY );' |   od --skip-bytes=25 -h --endian big
// bytes are: digits_count, weight, sign, scale, digit1, digit2, ..., last 2 bytes are trash

fn pg_decimal_to_u128(numeric: &PgNumeric) -> deserialize::Result<u128> {
    let (weight, scale, digits) = match *numeric {
        PgNumeric::Positive { weight, scale, ref digits } => (weight, scale, digits),
        PgNumeric::Negative { .. } => return Err(Box::from(format!("Negative is not supported in u128: {:#?}", numeric))),
        PgNumeric::NaN => return Err(Box::from(format!("NaN is not supported in u128: {:#?}", numeric))),
    };

    if scale != 0 {
        return Err(Box::from(format!("Nonzero scale is not supported in u128: {:#?}", numeric)));
    }

    if weight < 0 {
        return Err(Box::from(format!("Negative weight is not supported in u128: {:#?}", numeric)));
    }

    let mut result = 0u128;
    for digit in digits {
        result = result
            .checked_mul(10_000u128)
            .and_then(|res| res.checked_add(*digit as u128))
            .ok_or(Box::from(format!("Overflow in Pgnumeric to u128 (digits phase): {:#?}", numeric)) as Box<StdError + Send + Sync>)?;
    }

    let correction_exp = 4 * ((weight as u32) - (digits.len() as u32) + 1);
    // Todo - checked by using iteration;
    let pow = 10u128.pow(correction_exp);
    let result = result
        .checked_mul(pow)
        .ok_or(Box::from(format!("Overflow in Pgnumeric to u128 (correction phase): {:#?}", numeric)) as Box<StdError + Send + Sync>)?;
    Ok(result)
}

fn u128_to_pg_decimal(value: u128) -> PgNumeric {
    let digits = ToBase10000(Some(value)).collect::<Vec<_>>();
    let weight = digits.len() as i16 - 1;
    let mut digits: Vec<i16> = digits.into_iter().skip_while(|digit| *digit == 0).collect();
    digits.reverse();

    PgNumeric::Positive { digits, scale: 0, weight }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct PgBinary(String);

    impl Into<PgNumeric> for PgBinary {
        fn into(self) -> PgNumeric {
            let bytes: Vec<i16> = self.0.split(" ").map(|x| i16::from_str_radix(x, 16).unwrap()).collect();
            let weight = bytes[1];
            let scale = bytes[3];
            let digits = bytes[4..].to_vec();

            PgNumeric::Positive {
                weight: weight,
                scale: scale as u16,
                digits: digits,
            }
        }
    }

    // psql -U postgres -d challenge -c 'COPY ( SELECT CAST (34534 AS NUMERIC) ) TO STDOUT WITH ( FORMAT BINARY );' |   od --skip-bytes=25 -h --endian big

    #[test]
    fn test_conversions() {
        let cases = [
            ("0003 0006 0000 0000 03e8 0000 03e8", 1000000010000000000000000000u128),
            (
                "0009 0008 0000 0000 0003 1571 0005 0000 03e8 1103 1a94 0003 1296",
                354890005000010004355680400034758u128,
            ),
            ("0000 0000 0000 0000", 0u128),
            ("0001 0000 0000 0000 0001", 1u128),
            ("0001 0000 0000 0000 0002", 2u128),
            ("0001 0000 0000 0000 000a", 10u128),
            ("0001 0000 0000 0000 270f", 9999u128),
            ("0001 0001 0000 0000 0001", 10000u128),
            ("0002 0001 0000 0000 0001 0001", 10001u128),
            ("0002 0001 0000 0000 0001 0457", 11111u128),
            ("0002 0001 0000 0000 15b3 15b3", 55555555u128),
            ("0002 0001 0000 0000 270f 270f", 99999999u128),
            ("0003 0004 0000 0000 04d5 268f 095e", 12379871239800000000u128),
            (
                "000a 0009 0000 0000 0154 0b07 1a24 03aa 121a 18c1 11ff 10dd 1aa5 05ae",
                340282366920938463463374607431768211454u128,
            ),
            (
                "000a 0009 0000 0000 0154 0b07 1a24 03aa 121a 18c1 11ff 10dd 1aa5 05af",
                // u128 max value
                340282366920938463463374607431768211455u128,
            ),
        ];
        for case in cases.into_iter() {
            let (binary, number) = case.clone();
            let binary: PgBinary = PgBinary(binary.to_string());
            let pg_num: PgNumeric = binary.into();
            assert_eq!(pg_num, u128_to_pg_decimal(number), "u128 -> PgDecimal");
            assert_eq!(number, pg_decimal_to_u128(&pg_num).unwrap(), "PgDecimal -> u128");
        }
    }
}
