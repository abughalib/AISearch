# AI Search

AI Document Search Application.

## Requirments

- [Rust compiler](https://rustup.rs/)
- [Nvidia Cuda](https://developer.nvidia.com/cuda-downloads/)
- [PostgreSQL 12+](https://www.postgresql.org/download/)
- [PgVector](https://github.com/pgvector/pgvector)
  
### Windows

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

- Websocket (For Inferencing)

```css
ws://<machine_ip>:8000
```

## UI

[AISearchUI](https://github.com/abughalib/AISearchUI)
