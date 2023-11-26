use askama::Template;

use crate::database::model::book::Book;

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
#[template(path = "book/create-book.html")]
pub struct AddNewBookTemplate {}

#[derive(Default, Template)]
#[template(path = "book/add-book-owner.html")]
pub struct AddBookOwnerTemplate {}

#[derive(Default, Template)]
#[template(path = "dashboard.html")]
pub struct DashboardTemplate {}

#[derive(Default, Template)]
#[template(path = "book/books.html")]
pub struct BookListsBookTemplate<'a> {
    pub books: &'a [Book],
}

#[derive(Default, Template)]
#[template(path = "book/edit-book.html")]
pub struct EditBookTemplate<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub description: &'a str,
}
