use askama::Template;

use crate::database::model::{book::Book, category::Category, record::Record};

#[derive(Default, Template)]
#[template(path = "register.html")]
pub struct RegisterTemplate {}

#[derive(Default, Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {}

#[derive(Default, Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {}

#[derive(Default, Template)]
#[template(path = "not-found.html")]
pub struct NotFoundTemplate {}

#[derive(Default, Template)]
#[template(path = "dashboard.html")]
pub struct DashboardTemplate {}

#[derive(Default, Template)]
#[template(path = "book/create-book.html")]
pub struct AddNewBookTemplate {
    pub is_first_time: bool,
}

#[derive(Default, Template)]
#[template(path = "book/add-book-owner.html")]
pub struct AddBookOwnerTemplate {}

#[derive(Default, Template)]
#[template(path = "book/books.html")]
pub struct BookListsBookTemplate<'a> {
    pub books: &'a [Book],
}

#[derive(Default, Template)]
#[template(path = "book/edit-book.html")]
pub struct EditBookTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub is_can_delete: bool,
}

#[derive(Default, Template)]
#[template(path = "category/categories.html")]
pub struct CategoryListsTemplate<'a> {
    pub categories: &'a [Category],
}

#[derive(Default, Template)]
#[template(path = "category/create-category.html")]
pub struct AddNewCategoryTemplate {
    pub id: String,
}

#[derive(Default, Template)]
#[template(path = "category/edit-category.html")]
pub struct EditCategoryTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
}

pub struct RecordWithRupiah {
    pub record: Record,
    pub amount_in_rupiah: String,
    pub formatted_date: String,
}

#[derive(Default, Template)]
#[template(path = "record/records.html")]
pub struct RecordListsTemplate<'a> {
    pub records: &'a [RecordWithRupiah],
}

#[derive(Default, Template)]
#[template(path = "record/create-record.html")]
pub struct AddRecordTemplate<'a> {
    pub id: String,
    pub categories: &'a [Category],
}

#[derive(Default, Template)]
#[template(path = "record/edit-record.html")]
pub struct EditRecordTemplate<'a> {
    pub id: String,
    pub notes: String,
    pub category_id: String,
    pub amount: f32,
    pub categories: &'a [Category],
}
