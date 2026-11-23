//! Datas e horas da API com a crate `time`.
//!
//! O `chrono` saiu em 23/11/2026 (RUSTSEC-2026-0543, sem manutenção). Os
//! formatos de antes continuam: RFC 3339 nos textos de data e hora e
//! `AAAA-MM-DD` nas datas.

use time::format_description::well_known::Rfc3339;
use time::macros::format_description;
use time::{Date, Month, OffsetDateTime};

/// Agora, em UTC.
pub fn now() -> OffsetDateTime {
    OffsetDateTime::now_utc()
}

/// Segundos desde 01/01/1970 (UTC).
pub fn unix_now() -> i64 {
    now().unix_timestamp()
}

/// Agora em RFC 3339 (`2026-11-23T21:30:00.123456Z`).
pub fn now_rfc3339() -> String {
    now().to_rfc3339()
}

/// Data no formato `AAAA-MM-DD`.
pub fn parse_date(value: &str) -> Result<Date, time::error::Parse> {
    Date::parse(value, format_description!("[year]-[month]-[day]"))
}

/// `to_rfc3339()` como no `chrono`, para os pontos que devolvem data e hora em texto.
pub trait Rfc3339Ext {
    fn to_rfc3339(&self) -> String;
}

impl Rfc3339Ext for OffsetDateTime {
    fn to_rfc3339(&self) -> String {
        self.format(&Rfc3339).unwrap_or_default()
    }
}

/// Soma e subtração de meses com a regra do `chrono`: quando o dia não existe
/// no mês de destino, vale o último dia dele (31/01 + 1 mês = 28 ou 29/02).
pub trait MonthsExt: Sized {
    fn checked_add_months(self, months: u32) -> Option<Self>;
    fn checked_sub_months(self, months: u32) -> Option<Self>;
}

impl MonthsExt for Date {
    fn checked_add_months(self, months: u32) -> Option<Self> {
        shift_months(self, i64::from(months))
    }

    fn checked_sub_months(self, months: u32) -> Option<Self> {
        shift_months(self, -i64::from(months))
    }
}

fn shift_months(date: Date, months: i64) -> Option<Date> {
    let index = i64::from(date.year()) * 12 + i64::from(u8::from(date.month())) - 1 + months;
    let year = i32::try_from(index.div_euclid(12)).ok()?;
    let month = Month::try_from(u8::try_from(index.rem_euclid(12) + 1).ok()?).ok()?;
    let day = date.day().min(month.length(year));
    Date::from_calendar_date(year, month, day).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::{date, datetime};

    #[test]
    fn month_arithmetic_clamps_like_chrono() {
        assert_eq!(
            date!(2026 - 01 - 31).checked_add_months(1),
            Some(date!(2026 - 02 - 28))
        );
        assert_eq!(
            date!(2028 - 01 - 31).checked_add_months(1),
            Some(date!(2028 - 02 - 29))
        );
        assert_eq!(
            date!(2026 - 11 - 01).checked_add_months(2),
            Some(date!(2027 - 01 - 01))
        );
        assert_eq!(
            date!(2026 - 03 - 31).checked_sub_months(1),
            Some(date!(2026 - 02 - 28))
        );
        assert_eq!(
            date!(2026 - 11 - 23).checked_sub_months(18 * 12),
            Some(date!(2008 - 11 - 23))
        );
    }

    #[test]
    fn formats_match_the_previous_api() {
        assert_eq!(
            datetime!(2026-11-23 21:30:00.123456 UTC).to_rfc3339(),
            "2026-11-23T21:30:00.123456Z"
        );
        assert_eq!(parse_date("1990-05-10").unwrap(), date!(1990 - 05 - 10));
        assert!(parse_date("10/05/1990").is_err());
        assert_eq!(date!(2026 - 11 - 23).to_string(), "2026-11-23");
    }
}
