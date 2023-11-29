use chrono::serde::ts_milliseconds;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::prelude::FromRow;
use sqlx::Row;
use ulid::{serde::ulid_as_u128, Ulid};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Category {
    #[serde(with = "ulid_as_u128")]
    pub id: Ulid,
    #[serde(with = "ulid_as_u128")]
    pub book_id: Ulid,
    pub name: String,
    pub description: String,
    pub is_expense: bool,

    #[serde(with = "ts_milliseconds")]
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Category {
    pub fn new(name: &str, desc: &str, is_expense: bool, book_id: Ulid) -> Self {
        let id = ulid::Ulid::new();
        let created_at = chrono::offset::Utc::now();
        Self {
            id,
            name: (&name).to_string(),
            description: (&desc).to_string(),
            created_at,
            updated_at: None,
            deleted_at: None,
            book_id,
            is_expense,
        }
    }

    pub fn batch(book_id: Ulid) -> Vec<Self> {
        let predefined_categories = [
            ("Salary/Wages", "Your regular income from your job.", false),
            (
                "Freelance/Contract Work",
                "Income from any freelance or contract work you may do.",
                false,
            ),
            (
                "Side Hustle",
                "Income from any side businesses or projects you're involved in.",
                false,
            ),
            (
                "Investment Income",
                "Dividends, interest, or other income generated from investments.",
                false,
            ),
            (
                "Gifts/Donations",
                "Any money received as gifts or donations.",
                false,
            ),
            (
                "Rental Income",
                "If you earn income from renting out property.",
                false,
            ),
            ("Rent/Mortgage", "Monthly rent", true),
            ("Property Taxes", "Property tax payments", true),
            ("Home Insurance", "Insurance for your home", true),
            ("Electricity", "Monthly electricity bill", true),
            ("Water", "Monthly water bill", true),
            ("Gas", "Monthly gas bill", true),
            ("Internet/Phone", "Internet and phone bills", true),
            ("Car Payment", "Monthly car loan payment", true),
            ("Gas", "Gas expenses for your vehicle", true),
            ("Insurance", "Vehicle insurance", true),
            ("Public Transportation", "Public transportation costs", true),
            ("Food", "Grocery expenses", true),
            ("Household Supplies", "Expenses for household items", true),
            ("Health Insurance", "Health insurance premiums", true),
            ("Medications", "Costs for medications", true),
            ("Doctor's Visits", "Medical check-up expenses", true),
            ("Dining Out", "Expenses for dining out", true),
            ("Movies", "Entertainment expenses for movies", true),
            ("Subscriptions", "(Netflix, Spotify, etc.)", true),
            (
                "Credit Card Payments",
                "Payments towards credit card balances",
                true,
            ),
            ("Loan Payments", "Monthly loan payments", true),
            ("Haircuts", "Cost of haircuts", true),
            ("Toiletries", "Expenses for toiletries", true),
            (
                "Emergency Fund Contributions",
                "Contributions to your emergency fund",
                true,
            ),
            ("Retirement Savings", "Savings for retirement", true),
            ("Tuition", "Education tuition fees", true),
            ("Books", "Costs for educational books", true),
            ("Courses", "Expenses for additional courses", true),
            (
                "Miscellaneous",
                "Any other expenses not covered in the above categories.",
                true,
            ),
        ];

        predefined_categories
            .iter()
            .map(|&(name, description, is_expense)| {
                Self::new(name, description, is_expense, book_id)
            })
            .collect()
    }
}

impl FromRow<'_, PgRow> for Category {
    fn from_row(row: &'_ PgRow) -> Result<Self, sqlx::Error> {
        let id: [u8; 16] = row.get("id");
        let book_id: [u8; 16] = row.get("book_id");
        let is_expense: bool = row.get("is_expense");
        let name: String = row.get("name");
        let description: String = row.get("description");
        let created_at: DateTime<Utc> = row.get("created_at");
        let updated_at: Option<DateTime<Utc>> = match row.try_get("updated_at") {
            Ok(v) => v,
            Err(_) => None,
        };
        let deleted_at: Option<DateTime<Utc>> = match row.try_get("deleted_at") {
            Ok(v) => v,
            Err(_) => None,
        };

        let res: Category = Self {
            id: Ulid::from_bytes(id),
            created_at,
            updated_at,
            deleted_at,
            name,
            description,
            book_id: Ulid::from_bytes(book_id),
            is_expense,
        };
        Ok(res)
    }
}
