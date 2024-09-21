use std::env;
use std::path::PathBuf;

const INFERENCING_MODEL_PATH: &'static str = "PHI2_QUANTIZED_PATH";
const EMBEDDING_MODEL_PATH: &'static str = "EMBEDDING_MODEL_PATH";
const GENAI_DB_URL: &'static str = "GENAI_DB_URL";
const AZURE_AI_KEY: &'static str = "AZURE_OPENAI_KEY";
const ST_EMBEDDING_MODEL_PATH: &'static str = "ST_EMBEDDING_MODEL_PATH";
const ST_INFERENCING_MODEL_PATH: &'static str = "SAFETENSOR_MODEL_DIR";

pub fn get_inferencing_model_path() -> PathBuf {
    if let Ok(path) = env::var(&INFERENCING_MODEL_PATH) {
        return PathBuf::from(path);
    }
    panic!("{INFERENCING_MODEL_PATH} not set in environment variables");
}

pub fn get_azureai_api_key() -> String {
    if let Ok(res) = env::var(AZURE_AI_KEY) {
        return res;
    }

    panic!("{AZURE_AI_KEY} not set in environment variables");
}

pub fn get_pgurl() -> String {
    if let Ok(conn_url) = env::var(GENAI_DB_URL) {
        return conn_url;
    }
    panic!("{GENAI_DB_URL} not set in environment variables")
}

pub fn get_embedding_model_path() -> PathBuf {
    if let Ok(path) = env::var(&EMBEDDING_MODEL_PATH) {
        return PathBuf::from(path);
    };
    panic!("{EMBEDDING_MODEL_PATH} not set in environment variables")
}

pub fn safetensor_embedding_model_path() -> PathBuf {
    match env::var(ST_EMBEDDING_MODEL_PATH) {
        Ok(res) => return path_exists(&res),
        Err(e) => {
            panic!("Set {ST_EMBEDDING_MODEL_PATH} environment variable: {e}")
        }
    }
}

pub fn safetensor_model_path() -> PathBuf {
    match env::var(ST_INFERENCING_MODEL_PATH) {
        Ok(res) => {
            return path_exists(&res);
        }
        Err(e) => {
            panic!("Set {ST_INFERENCING_MODEL_PATH} environment variable: {e}")
        }
    }
}

pub fn path_exists(dir: &String) -> PathBuf {
    let dir = PathBuf::from(dir);

    if dir.exists() {
        return dir;
    }

    panic!("Dir: {:?} doesn't exists", dir);
}

pub fn get_app_config_path() -> PathBuf {
    let path = PathBuf::from(
        env::var("APP_CONFIG_PATH").expect("APP_CONFIG_PATH not set in environment variables"),
    );
    if path.exists() {
        return path;
    }
    panic!("Dir: {:?} doesn't exists", path);
}
