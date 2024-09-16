use std::path::PathBuf;

use crate::local::database;
use crate::local::inferencing;
use crate::upload::{learn_from_pdf, learn_from_text};

use futures::SinkExt;
use futures::StreamExt;
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::task;
use warp::http::Response;
use warp::Buf;

#[derive(Debug, Deserialize)]
struct WebSocketMessage {
    table_name: String,
    session_id: String,
    sentence: String,
    deployment_type: String,
    deployment_model: String,
    max_similar_search: usize,
    upper_chunk: i32,
    lower_chunk: i32,
    minimum_score: f32,
}

pub async fn home() -> Result<impl warp::Reply, warp::Rejection> {
    Ok(warp::reply::with_status(
        "Nothing Here",
        warp::http::StatusCode::OK,
    ))
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UploadForm {
    pub table_name: String,
    pub files: Vec<String>,
}

pub async fn handle_upload(
    form: warp::multipart::FormData,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut parts = form.into_stream();
    let mut file_paths: Vec<String> = Vec::new();
    let mut table_name: Option<String> = None;

    while let Ok(Some(part)) = parts.try_next().await {
        if part.name() == "files" {
            let filename: &str = part.filename().unwrap_or("unknown");
            let filepath = format!("upload_path/{filename}");

            let mut file = File::create(&filepath)
                .await
                .expect("Unable to write to file");

            let mut stream = part.stream();

            while let Ok(Some(chunk)) = stream.try_next().await {
                file.write_all(&mut chunk.chunk())
                    .await
                    .expect("Unable to write to file");
            }
            file_paths.push(filepath);
        } else {
            let name = part.name().to_string();
            let value = part
                .stream()
                .try_fold(Vec::new(), |mut acc, buf| async move {
                    acc.extend_from_slice(buf.chunk());
                    Ok(acc)
                })
                .await;

            if name == String::from("table_name") {
                if let Ok(value) = value {
                    table_name = Some(String::from_utf8_lossy(&value).to_string())
                }
            }
        }
    }

    println!("table_name: {:?}\nFile_names: {:?}", table_name, file_paths);

    if let Some(table_name) = table_name {
        for file_name in file_paths {
            if file_name.ends_with("txt") {
                let table_name_clone = table_name.clone();
                task::spawn(async move {
                    let _ = learn_from_text(&table_name_clone, &PathBuf::from(file_name)).await;
                });
            } else if file_name.ends_with("pdf") {
                let table_name_clone = table_name.clone();
                task::spawn(async move {
                    let _ = learn_from_pdf(&table_name_clone, &PathBuf::from(file_name)).await;
                });
            } else {
                println!("File type not supported: {file_name}");
            }
        }
    }

    Ok(warp::reply::with_status(
        "File is Processing",
        warp::http::StatusCode::OK,
    ))
}

#[derive(Serialize, Deserialize)]
pub struct TableCreate {
    table_name: String,
}

pub async fn create_new_table(
    table_create: TableCreate,
) -> Result<impl warp::Reply, warp::Rejection> {
    if database::create_table(&table_create.table_name)
        .await
        .is_ok()
    {
        Ok(Response::builder()
            .status(200)
            .body::<String>("Table created successfully".into())
            .unwrap())
    } else {
        Ok(Response::builder()
            .status(502)
            .body("Failed to create table".into())
            .unwrap())
    }
}

pub async fn delete_table(table_create: TableCreate) -> Result<impl warp::Reply, warp::Rejection> {
    if database::delete_table(&table_create.table_name)
        .await
        .is_ok()
    {
        Ok(Response::builder()
            .status(200)
            .body::<String>(format!("Table Deleted: {}", &table_create.table_name))
            .unwrap())
    } else {
        Ok(Response::builder()
            .status(502)
            .body("Failed to create table".into())
            .unwrap())
    }
}

pub async fn get_all_search_base() -> Result<impl warp::Reply, warp::Rejection> {
    match database::list_search_tables().await {
        Ok(tables) => {
            let json_resp = serde_json::to_string(&tables).unwrap();
            Ok(Response::builder()
                .status(200)
                .body::<String>(json_resp.into())
                .unwrap())
        }
        Err(_) => Ok(Response::builder()
            .status(500)
            .body("Failed to retrieve tables".into())
            .unwrap()),
    }
}

pub async fn ws_handler(ws: warp::ws::Ws) -> Result<impl warp::Reply, warp::Rejection> {
    Ok(ws.on_upgrade(move |socket| client_connection(socket)))
}

pub async fn client_connection(ws: warp::ws::WebSocket) {
    let (mut tx, mut rx) = ws.split();

    let mut query_model = inferencing::ModelQuery::new();

    while let Some(result) = rx.next().await {
        match result {
            Ok(msg) => {
                if let Ok(str_msg) = msg.to_str() {
                    if let Ok(socket_message) = serde_json::from_str::<WebSocketMessage>(str_msg) {
                        if socket_message.table_name.is_empty() {
                            let _ = tx
                                .send(warp::ws::Message::text("No Knowledge Base Selected"))
                                .await;
                            continue;
                        }

                        if let Ok(embed_value) = query_model
                            .get_embeddings(
                                &socket_message.table_name,
                                &socket_message.sentence.trim(),
                                socket_message.max_similar_search,
                                socket_message.lower_chunk,
                                socket_message.upper_chunk,
                                socket_message.minimum_score,
                            )
                            .await
                        {
                            let _ = query_model
                                .answer_with_context(
                                    &mut tx,
                                    &socket_message.sentence.trim(),
                                    &socket_message.session_id.trim(),
                                    &socket_message.deployment_type,
                                    &socket_message.deployment_model,
                                    embed_value,
                                )
                                .await;
                        }
                    } else {
                        println!("{:?}", serde_json::from_str::<WebSocketMessage>(str_msg));
                        let _ = tx
                            .send(warp::ws::Message::text("Cannot Parse the Input"))
                            .await;
                    }
                }
            }
            Err(_e) => {
                break;
            }
        }
    }
}
