// src/templates.rs

use askama::Template;
use crate::models::{Post, User};

#[derive(Template)]
#[template(path = "register.html")]
pub struct RegisterTemplate {
    pub error: Option<String>,
}

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {
    pub error: Option<String>,
}

#[derive(Template)]
#[template(path = "profile.html")]
pub struct ProfileTemplate {
    pub user: User,
    pub current_user: Option<User>,
    pub is_vulnerable: bool,
}

#[derive(Template)]
#[template(path = "search.html")]
pub struct SearchTemplate {
    pub query: String,
    pub posts: Vec<(Post, String)>,
    pub current_user: Option<User>,
    pub is_vulnerable: bool,
}
