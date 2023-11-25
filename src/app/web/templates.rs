use askama::Template;

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
#[template(path = "protected.html")]
pub struct ProtectedTemplate {}
