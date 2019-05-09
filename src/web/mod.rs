use crate::db::Db;
use askama::Template;
use rouille::{router, Response};
use std::io;
use std::net::ToSocketAddrs;
use std::sync::Arc;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate;

pub(crate) fn serve<A>(addr: A, db: Db) -> !
where
    A: ToSocketAddrs,
{
    let db = Arc::new(db);
    rouille::start_server(addr, move |request| {
        let db = db.clone();
        rouille::log(request, io::stdout(), || {
            router!(request,
                (GET) (/) => {
                    Response::html(IndexTemplate.render().unwrap())
                },
                _ => Response::empty_404(),
            )
        })
    });
}
