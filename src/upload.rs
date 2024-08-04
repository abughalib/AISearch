use crate::database::bulk_insert_split_chunks;
use anyhow::{Context, Error, Result};
use chrono::Utc;
use serde_json::json;
use std::path::PathBuf;
use tokio::{
    fs,
    io::{AsyncBufReadExt, BufReader},
};

#[derive(Clone, Debug, PartialEq)]
pub enum FileType {
    PDF,
    Text,
}

pub async fn learn_from_pdf(table_name: &str, file_path: &PathBuf) -> Result<()> {
    let bytes = fs::read(file_path.clone()).await?;

    let extracted_file = pdf_extract::extract_text_from_mem(&bytes)?;

    let file_name = file_path
        .file_name()
        .context("Unable to get file name")?
        .to_str()
        .context("Unable to convert file name to String")?;

    println!("Processing PDF File Name: {file_name}");

    bulk_insert_split_chunks(
        table_name,
        file_name,
        &extracted_file,
        json!({
            "source": file_name,
            "upload_time": Utc::now().to_string()
        }),
    )
    .await
    .map_err(Error::msg)?;

    println!("Uploaded PDF File Name: {file_name}");

    Ok(())
}

pub async fn learn_from_text(table_name: &str, file_path: &PathBuf) -> Result<()> {
    let file_name = file_path
        .file_name()
        .context("Unable to get file name")?
        .to_str()
        .context("Unable to convert file name to String")?;

    println!("Processing PDF File Name: {file_name}");

    let file = fs::File::open(&file_path).await?;

    let reader = BufReader::new(file);

    let mut reader_lines = reader.lines();

    let mut contents: String = String::new();

    while let Some(line) = reader_lines.next_line().await? {
        contents += &line;
        contents.push('\n');
    }

    bulk_insert_split_chunks(
        table_name,
        file_name,
        &contents,
        json!({
            "source": file_name,
            "upload_time": Utc::now().to_string()
        }),
    )
    .await
    .map_err(Error::msg)?;

    println!("Uploaded PDF File Name: {file_name}");

    Ok(())
}
