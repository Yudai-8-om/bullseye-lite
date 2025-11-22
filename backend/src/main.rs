// use axum::http::StatusCode;
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::Path,
    extract::State,
    response::Response,
    routing::get,
    Json, Router,
};
use db::{establish_connection_pool, lookup_exchange};
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use errors::BullsEyeError;
use http::Method;
use models::earnings_model::EarningsReport;
use models::forecast_models::Forecasts;
use models::metrics_model::CurrentMetrics;
use models::returning_model::ReturningModel;
use rand::Rng;
use tokio::time::{self, Duration};
use tower_http::cors::CorsLayer;

mod calculate;
mod db;
mod errors;
mod helper;
mod metrics;
mod models;
mod query;
mod schema;
mod services;

async fn search(
    State(pool): State<Pool<ConnectionManager<PgConnection>>>,
    Path(ticker): Path<String>,
) -> Result<Json<ReturningModel>, BullsEyeError> {
    let exchange = lookup_exchange(&ticker);
    let conn = &mut pool.get().unwrap();
    let company = services::get_company(&ticker, &exchange, conn).await?;
    let forecast = Forecasts::load_by_id(company.id, conn)?;
    let earnings_update_needed = forecast.is_earnings_update_needed();
    if earnings_update_needed {
        let latest_earnings = EarningsReport::latest_quarter_data_if_existed(company.id, conn)?;
        let all_earnings = match latest_earnings {
            Some(earnings) => earnings.quarter_str == 3,
            None => true,
        };
        if all_earnings {
            services::update_earnings_all(company.id, &ticker, &exchange, conn).await?;
            services::update_metrics_annual(company.id, conn)?;
        } else {
            services::update_earnings_ttm(company.id, &ticker, &exchange, conn).await?;
            services::update_metrics_ttm(company.id, conn)?;
        }
    } else {
        let regular_update_needed = forecast.is_regular_update_needed();
        if regular_update_needed {
            services::update_regular(company.id, &ticker, &exchange, conn).await?;
        }
        services::update_metrics_annual(company.id, conn)?;
    }
    let all_metrics = CurrentMetrics::load_by_id(company.id, conn)?;
    let all_forecasts = Forecasts::load_by_id(company.id, conn)?;
    Ok(Json(ReturningModel::new(
        company,
        all_metrics,
        all_forecasts,
    )))
}
async fn list_all(
    State(pool): State<Pool<ConnectionManager<PgConnection>>>,
) -> Result<Json<Vec<ReturningModel>>, BullsEyeError> {
    let conn = &mut pool.get().unwrap();
    let all_companies: Vec<ReturningModel> = services::get_all_companies(conn)?;
    Ok(Json(all_companies))
}

async fn get_stock_price(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket))
}

async fn handle_socket(mut socket: WebSocket) {
    let mut interval = time::interval(Duration::from_millis(100));
    loop {
        interval.tick().await;
        let sample = {
            let mut rng = rand::rng();
            100.0 + (rng.random::<f64>() * 50.0)
        };
        if socket
            .send(Message::Text(format!("Price: {}", sample).into()))
            .await
            .is_err()
        {
            return;
        }
    }
}

#[tokio::main]
async fn main() {
    // build our application with a single route
    let allowed_origins = vec![
        "http://192.168.1.12".parse().unwrap(),
        "http://192.168.1.12:5173".parse().unwrap(), // only dev
    ];
    let cors = CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods([Method::GET]);
    let pool = establish_connection_pool().unwrap();
    let app = Router::new()
        .route("/screener", get(list_all))
        .route("/companies/{ticker}", get(search))
        .route("/ws", get(get_stock_price))
        .with_state(pool)
        .layer(cors);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
