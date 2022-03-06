use actix_web::{http::Method, web, App, HttpRequest, HttpResponse, HttpServer};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct ReqObj {
    x: i32,
    y: i32,
}

#[derive(Debug, Deserialize, Serialize)]
struct ResObj {
    result: i32,
}

async fn add(req: web::Json<ReqObj>) -> HttpResponse {
    HttpResponse::Ok().json(ResObj {
        result: req.x + req.y,
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(
                web::resource("/health").to(|req: HttpRequest| match *req.method() {
                    Method::GET => HttpResponse::Ok(),
                    _ => HttpResponse::NotFound(),
                }),
            )
            .service(web::resource("/add").route(web::post().to(add)))
    })
    .bind(("0.0.0.0", 5000))?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::body::to_bytes;
    use actix_web::dev::{Service, ServiceResponse};
    use actix_web::{http, test, web, App};
    use std::str;

    #[actix_web::test]
    async fn test_add() {
        let app = test::init_service(
            App::new().service(web::resource("/add").route(web::post().to(add))),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/add")
            .set_json(&ReqObj { x: 1, y: 2 })
            .to_request();

        let res: ServiceResponse = app.call(req).await.unwrap();

        assert_eq!(res.status(), http::StatusCode::OK);

        let body: ResObj = serde_json::from_str(
            str::from_utf8(&to_bytes(res.into_body()).await.unwrap()).unwrap(),
        )
        .unwrap();
        assert_eq!(body.result, 3);
    }
}
