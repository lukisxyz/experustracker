use askama::Template;

#[derive(Default, Template)]
#[template(path = "register.html")]
pub struct RegisterTemplate {}

#[derive(Default, Template)]
#[template(path = "not-found.html")]
pub struct NotFoundTemplate {}
