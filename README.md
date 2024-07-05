# AI Search

AI Document Search Application.


# Build

```sh
cargo build --release
```

# Setup
**PHI2 Inferencing Model Q4**
```bash
export PHI2_QUANTIZED_PATH=<inferecing_model_path>
```
**Embedding Model Path**
```bash
export BAAI_PATH=<embedding_model_path>
```
**PostgreSQL DB**
```bash
export GENAI_DB_URL=<postgresql_conn_url>
```
**Azure AI Service**
```bash
export AZURE_AI_KEY=<azure_api_key>
```

# API Routes

**Websocket (For Inferencing)**
```css
ws://<machine_ip>:8000
```