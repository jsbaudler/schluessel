use actix_web::{middleware::Logger, post, web, App, HttpResponse, HttpServer, Responder};
use env_logger::Env;
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
            <title>🔑 Schluessel 🔑</title>
            <style>
                body {
                    display: flex;
                    justify-content: center;
                    align-items: center;
                    height: 100vh;
                }

                .form-signin {
                    width: 400px;
                    text-align: center;
                }

                .form-control {
                    margin-bottom: 20px;
                    width: 392px;
                    font-size: 20px;
                }

                .btn {
                    width: 100%;
                    color: white;
                    background-color: #007bff;
                    border: none;
                    padding: 20px;
                    font-size: 20px;
                }

                .btn:hover {
                    background-color: #0056b3;
                }
            </style>
        </head>

        <body>
            <form action="/authenticate" method="post" class="form-signin">
                <h1>This is Schluessel 🔑</h1>
                <div class="form-group">
                    <input type="password" id="password" name="password" class="form-control" placeholder="Password"
                        required>
                </div>
                <button class="btn" type="submit">Sign in</button>
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
        let rendered = generate_services_html(&instances);

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

    log::info!("Domain: {0:?} with {1:?} successfully registered.", form.domain, form.services);

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
            <style>
                .service {
                    width: 20%;
                    box-shadow: 0 4px 8px 0 rgba(0,0,0,0.2);
                    padding: 10px;
                    margin: 10px;
                    display: inline-block;
                    vertical-align: top;
                }

                .service h3 {
                    margin: 5px 0 15px 0;
                }

                .service button {
                    width: 100%;
                    color: white;
                    background-color: #007bff;
                    border: none;
                    padding: 20px;
                    font-size: 20px;
                    text-align: center;
                }

                .service button:hover {
                    background-color: #0056b3;
                }
            </style>
        </head>
        <body>
            <h1>Services</h1>
    "#,
    );

    for (domain, services) in service_map {
        html.push_str(&format!("<h2>{}</h2>", domain));

        for (service_name, service_url) in services {
            let action_url = format!("{}/generate_auth_cookie", service_url);
            html.push_str(&format!(
                r#"<div class="service">
                    <form action="{}" method="post" target="_blank">
                        <input type="hidden" name="shared_secret" value="{}">
                        <input type="hidden" name="service_url" value="{}">
                        <h3>{}</h3>
                        <button type="submit">Go to Service</button>
                    </form>
                </div>"#,
                action_url, shared_secret, service_url, service_name
            ));
        }
    }

    html.push_str(
        r#"
        </body>
        </html>
    "#,
    );

    html
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    // initialize logging
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let data = web::Data::new(AppState {
        registered_schloss_instances: Mutex::new(HashMap::new()),
    });

    // read all the env vars
    let http_host = env::var("HTTP_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let http_port = env::var("HTTP_PORT").unwrap_or_else(|_| "8080".to_string());

    let bind_address = format!("{}:{}", http_host, http_port);

    // set the schluessel version
    let schluessel_version = env::var("SCHLUESSEL_VERSION")
    .or_else(|_| env::var("CARGO_PKG_VERSION"))
    .unwrap_or_else(|_| "0.0.0-dev (not set)".to_string());

    // print out some basic info about the server
    log::info!("Starting Schluessel v{schluessel_version}");
    log::info!("Serving at {http_host}:{http_port}");

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(data.clone())
            .route("/", web::get().to(index))
            .service(authenticate)
            .service(register_schloss)
    })
    .bind(bind_address)?
    .run()
    .await
}
