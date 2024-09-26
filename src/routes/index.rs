use actix_web::{
    body, dev::Server, get, http::StatusCode, web, App, Error, HttpRequest, HttpResponse,
    HttpServer, Responder,
};
use futures::{future::ok, stream::once};
use ssr_rs::Ssr;
use std::collections::HashMap;
use tera::{Context, Tera};

use crate::AppState;

#[get("/")]
pub async fn index(req: HttpRequest, data: web::Data<AppState>) -> impl Responder {
    let props = format!(
        r##"{{
            "location": "{}",
            "context": {{}}
        }}"##,
        req.uri()
    );

    let mut context = Context::new();
    context.insert("foo", "123");

    let source = data.js_source.lock().unwrap();

    println!("{}", source);

    let response_body = Ssr::render_to_string(&source, "SSR", Some(&props));

    let body = once(ok::<_, Error>(web::Bytes::from(response_body)));

    HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .streaming(body)
}

async fn health_check() -> impl Responder {
    HttpResponse::Ok().finish()
}
