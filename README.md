# AI Search

AI Document Search Application.

## Requirments

- [Rust compiler](https://rustup.rs/)
- [Nvidia Cuda](https://developer.nvidia.com/cuda-downloads/)
- [PostgreSQL 12+](https://www.postgresql.org/download/)
- [PgVector](https://github.com/pgvector/pgvector)

### Windows

- If I find anything it would be here.

### Linux Specific

- `libssl-dev` and `postgresql-server-dev`

  ```sh
  sudo apt install libssl-dev postgresql-server-dev
  ```

  ```sh
  cargo build --release
  ```

## Setup

- [PHI2 Inferencing Model Q4](https://huggingface.co/Demonthos/dolphin-2_6-phi-2-candle)

  ```bash
  export PHI2_QUANTIZED_PATH=<inferecing_model_path>
  ```

- [Embedding Model Path](https://huggingface.co/BAAI/bge-large-en-v1.5)

  ```bash
  export EMBEDDING_MODEL_PATH=<embedding_model_path>
  ```

- PostgreSQL DB
  _Make sure the USER have superuser permission for the first time to create vector extension or do it yourself first_

  ```bash
  export GENAI_DB_URL=<postgresql_conn_url>
  ```

- Azure AI Service (For Azure OpenAI Model)

  ```bash
  export AZURE_AI_KEY=<azure_api_key>
  ```

## API Routes

For Testing Use cURL, [WebSocat](https://github.com/vi/websocat) or [Postman](https://www.postman.com/downloads/).

```bash
export HOST_IP="0.0.0.0" # Replace it with yours
export HOST_PORT="8000" # Replace it with yours
```

- Websocket (For Inferencing)
  - Deployment Type: LOCAL/AZURE ...
  - Deployment Model: PHI2/PHI3/GPT-4o ...
  - Max Similar Search: The number of maximum similar searches to get from DB.
  - Upper Chunks & Lower Chunks: Maximum row content to get from the search line.

  ```bash
  ./websocat http://$HOST_IP:$HOST_PORT/ws/ -d '{
    "table_name": "your_table_name",
    "session_id": "your_session_id",
    "sentence": "your_sentence",
    "deployment_type": "your_deployment_type",
    "deployment_model": "your_deployment_model",
    "max_similar_search": 10,
    "upper_chunks": 5,
    "lower_chunks": 3
  }'
  ```

- Search Bases

  ```bash
  curl -X POST "http://$HOST_IP:$HOST_PORT/search_bases"
  ```

- Create Table

  ```bash
  curl -X POST "http://$HOST_IP:$HOST_PORT/create_table" -H "Content-Type: application/json" -d '{
    "table_name": "testing"
  }'
  ```

- Upload File

  ```bash
  curl -X POST http://$HOST_IP:$HOST_PORT/handle_upload \
      -H "Content-Type: multipart/form-data" \
      -F "table_name=testing" \
      -F "files=@file.txt"
  ```

- Delete Table

  ```bash
  curl -X POST "http://$HOST_IP:$HOST_PORT/delete_table" -H "Content-Type: application/json" -d '{
    "table_name": "testing"
  }'
  ```

## UI

[AISearchUI](https://github.com/abughalib/AISearchUI)
