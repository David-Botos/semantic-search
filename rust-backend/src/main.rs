// src/main.rs
mod db;
mod embedding;

use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use anyhow::Result;
use log::{debug, error, info}; // Added debug level if needed for more detail
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct SearchQuery {
    query: String,
    limit: Option<i32>,
    latitude: Option<f64>,
    longitude: Option<f64>,
}

// Add Debug derive for easy logging if needed
#[derive(Serialize, Debug)] // <-- Added Debug
struct SearchResult {
    id: String,
    name: String,
    description: Option<String>,
    short_description: Option<String>,
    status: String,
    organization_name: Option<String>,
    similarity: f64,
    distance: Option<f64>, // Distance in meters
}

#[actix_web::get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("Semantic Search API is running")
}

#[actix_web::post("/search")]
async fn search(
    query: web::Json<SearchQuery>,
    app_data: web::Data<AppState>,
) -> impl Responder {
    let limit = query.limit.unwrap_or(50);
    info!(
        "Search request received: '{}', limit: {}, lat: {:?}, lng: {:?}",
        query.query, limit, query.latitude, query.longitude
    ); // Added limit to initial log

    let embedding_result =
        embedding::generate_embedding(&query.query, &app_data.embedding_model).await;
    let query_embedding = match embedding_result {
        Ok(embedding) => embedding,
        Err(e) => {
            error!("Embedding error: {}", e);
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": "Embedding generation failed"}));
        }
    };

    match search_services(
        &app_data.db_pool,
        &query_embedding,
        query.latitude,
        query.longitude,
        limit,
    )
    .await
    {
        Ok(results) => {
            // --- Logging added here ---
            info!(
                "Search successful for query '{}'. Found {} results.",
                query.query,
                results.len()
            );

            // Optional: Log details of each result at DEBUG level
            // This might be verbose if there are many results.
            // Ensure your logger is configured to show DEBUG level if you use this.
            if log::log_enabled!(log::Level::Debug) { // Check if debug logging is enabled
                for (i, result) in results.iter().enumerate() {
                    debug!(
                        "  Result {}: ID={}, Name='{}', Similarity={:.4}, Distance={:?}",
                        i + 1,
                        result.id,
                        result.name,
                        result.similarity,
                        result.distance
                    );
                }
            } else if let Some(first_result) = results.first() {
                 // If debug isn't enabled, log summary of the top result at INFO level
                 info!(
                     "Top result: ID={}, Name='{}', Similarity={:.4}, Distance={:?}",
                     first_result.id, first_result.name, first_result.similarity, first_result.distance
                 );
            }
            // --- End of logging ---

            HttpResponse::Ok().json(results)
        }
        Err(e) => {
            error!("Search failed: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({"error": "Search failed"}))
        }
    }
}

async fn search_services(
    pool: &db::PgPool,
    query_embedding: &[f32],
    latitude: Option<f64>,
    longitude: Option<f64>,
    limit: i32,
) -> Result<Vec<SearchResult>> {
    let client = pool.get().await?;
    let query_vector = pgvector::Vector::from(query_embedding.to_vec());
    let limit_i64: i64 = limit.into();

    let rows = if let (Some(lat), Some(lng)) = (latitude, longitude) {
        client.query(
            r#"
            WITH nearby_services AS (
                SELECT
                    s.id,
                    s.embedding_v2,
                    s.name,
                    s.description,
                    s.short_description,
                    s.status,
                    o.name as organization_name,
                    MIN(ST_Distance(l.geom, ST_MakePoint($2, $1)::geography)) as distance
                FROM service s
                JOIN service_at_location sal ON s.id = sal.service_id
                JOIN location l ON sal.location_id = l.id
                JOIN organization o ON s.organization_id = o.id
                WHERE s.embedding_v2 IS NOT NULL AND l.geom IS NOT NULL
                GROUP BY s.id, s.embedding_v2, s.name, s.description, s.short_description, s.status, o.name
                HAVING MIN(ST_Distance(l.geom, ST_MakePoint($2, $1)::geography)) <= 16093.44  -- 5 miles in meters
            )
            SELECT
                id,
                name,
                description,
                short_description,
                status,
                organization_name,
                (1 - (embedding_v2 <=> $3))::float8 as similarity,
                distance
            FROM nearby_services
            ORDER BY (1 - (embedding_v2 <=> $3)) DESC
            LIMIT $4
            "#,
            &[&lat, &lng, &query_vector, &limit_i64],
        ).await?
    } else {
        client.query(
            r#"
            SELECT
                s.id,
                s.name,
                s.description,
                s.short_description,
                s.status,
                o.name as organization_name,
                (1 - (s.embedding_v2 <=> $1))::float8 as similarity,
                NULL as distance
            FROM service s
            JOIN organization o ON s.organization_id = o.id
            ORDER BY (1 - (s.embedding_v2 <=> $1)) DESC
            LIMIT $2
            "#,
            &[&query_vector, &limit_i64],
        ).await?
    };

    let results = rows.iter().map(|row| SearchResult {
        id: row.get("id"),
        name: row.get("name"),
        description: row.get("description"),
        short_description: row.get("short_description"),
        status: row.get("status"),
        organization_name: row.get("organization_name"),
        similarity: row.get("similarity"),
        distance: row.try_get("distance").ok().flatten(), 
    }).collect();

    Ok(results)
}
struct AppState {
    db_pool: db::PgPool,
    embedding_model: embedding::EmbeddingModel,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Ensure RUST_LOG env var can be set, e.g., RUST_LOG=info or RUST_LOG=debug
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    info!("Starting Semantic Search Microservice");

    dotenv::dotenv().ok();

    let db_pool = db::connect().await.map_err(|e| {
        error!("Database connection failed: {}", e);
        std::io::Error::new(std::io::ErrorKind::Other, "Database connection failed")
    })?;

    let embedding_model = embedding::init_model().map_err(|e| {
        error!("Model initialization failed: {}", e);
        std::io::Error::new(std::io::ErrorKind::Other, "Model initialization failed")
    })?;

    let host = std::env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("SERVER_PORT").unwrap_or_else(|_| "8080".to_string());
    let port = port.parse::<u16>().unwrap_or(8080);

    info!("Server binding to {}:{}", host, port); // Added log for host/port

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
            )
            .app_data(web::Data::new(AppState {
                db_pool: db_pool.clone(),
                embedding_model: embedding_model.clone(),
            }))
            .service(index)
            .service(search)
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await
}