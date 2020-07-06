use chrono::{Datelike, DateTime, NaiveDate, Utc};

pub type Id = uuid::Uuid;
pub type Time = DateTime<Utc>;
pub type Date = NaiveDate;

pub trait DateExt {
	fn increment_date_by_months(&self, num_months: u16) -> Date;
}

impl DateExt for Date {
	fn increment_date_by_months(&self, num_months: u16) -> Date {
		let mut add_years: u32 = (num_months / 12) as u32;
		let mut add_months: u32 = (num_months % 12) as u32;
		
		let result_month: u32;
		
		let total_months = self.month() + (add_months as u32);
		if total_months > 12 {
			result_month = (total_months / 12);
			add_years += 1;
		} else {
			result_month = total_months;
		}
		
		let result_year: i32 = self.year() + add_years as i32;
		
		chrono::NaiveDate::from_ymd(result_year, result_month, self.day())
	}
}

