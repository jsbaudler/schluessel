use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::sync::Mutex;

async fn index() -> impl Responder {
    let rendered = r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <title>Schluessel</title>
            <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bootstrap@4.6.2/dist/css/bootstrap.min.css">
        </head>
        <body class="d-flex justify-content-center align-items-center vh-100">
            <form action="/authenticate" method="post" class="form-signin">
                <h1 class="h3 mb-3 font-weight-normal">Please sign in</h1>
                <div class="form-group">
                    <label for="password" class="sr-only">Password</label>
                    <input type="password" id="password" name="password" class="form-control" placeholder="Password" required>
                </div>
                <button class="btn btn-lg btn-primary btn-block" type="submit">Sign in</button>
            </form>
        </body>
        </html>
    "#;

    HttpResponse::Ok().body(rendered)
}

#[derive(Deserialize)]
struct AuthRequest {
    password: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct SchlossInstance {
    domain: String,
    services: Vec<(String, String)>,
}

struct AppState {
    registered_schloss_instances: Mutex<HashMap<String, Vec<(String, String)>>>,
}

#[post("/authenticate")]
async fn authenticate(form: web::Form<AuthRequest>, data: web::Data<AppState>) -> HttpResponse {
    let expected_password = env::var("PASSWORD").unwrap_or_else(|_| "password".to_string());

    if form.password == expected_password {
        // Retrieve the registered Schloss instances from the in-memory store
        let instances = data.registered_schloss_instances.lock().unwrap();

        // Generate the HTML content dynamically
        let rendered = generate_services_html(&*instances);

        HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(rendered)
    } else {
        HttpResponse::Unauthorized().body("Unauthorized")
    }
}

#[post("/register")]
async fn register_schloss(
    form: web::Json<SchlossInstance>,
    data: web::Data<AppState>,
) -> impl Responder {
    let mut instances = data.registered_schloss_instances.lock().unwrap();

    // Store the Schloss instance and its services in the in-memory store
    instances.insert(form.domain.clone(), form.services.clone());

    HttpResponse::Ok().json(form.into_inner()) // Return the registered instance as a response
}

fn generate_services_html(service_map: &HashMap<String, Vec<(String, String)>>) -> String {
    let shared_secret = env::var("SHARED_SECRET").unwrap_or("shared_secret".to_string());
    let mut html = String::new();

    html.push_str(
        r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <title>Services</title>
            <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bootstrap@4.6.2/dist/css/bootstrap.min.css">
        </head>
        <body class="container pt-5">
            <h1 class="mb-4">Services</h1>
            <div class="row">
    "#,
    );

    for (domain, services) in service_map {
        html.push_str(&format!("<h2 class='col-12 mt-5'>{}</h2>", domain));

        for (service_name, service_url) in services {
            html.push_str(&format!(
                r#"<div class="col-md-4">
                    <div class="card mb-4 shadow-sm">
                        <div class="card-body">
                            <form action="{}" method="post" target="_blank">
                                <input type="hidden" name="shared_secret" value="{}">
                                <input type="hidden" name="service_url" value="{}">
                                <h5 class="card-title">{}</h5>
                                <button type="submit" class="btn btn-primary">Go to Service</button>
                            </form>
                        </div>
                    </div>
                </div>"#,
                format!("{}/generate_auth_cookie", service_url),
                shared_secret,
                service_url,
                service_name
            ));
        }
    }

    html.push_str(
        r#"
            </div>
        </body>
        </html>
    "#,
    );

    html
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let data = web::Data::new(AppState {
        registered_schloss_instances: Mutex::new(HashMap::new()),
    });

    let http_host = env::var("HTTP_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let http_port = env::var("HTTP_PORT").unwrap_or_else(|_| "8080".to_string());

    let bind_address = format!("{}:{}", http_host, http_port);

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .route("/", web::get().to(index))
            .service(authenticate)
            .service(register_schloss)
    })
    .bind(bind_address)?
    .run()
    .await
}
