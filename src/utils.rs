use candle_core::Device;
use std::env;
use std::path::PathBuf;

pub enum MyResult<T> {
    Ok(T),
}

pub fn get_pg_url() -> String {
    match env::var("PHI2DB") {
        Ok(res) => {
            return res;
        }
        Err(e) => {
            panic!("Set PHI2DB environment variable: {e}");
        }
    }
}

pub fn infer_model_path() -> PathBuf {
    match env::var("QUANTIZED_MODEL_DIR") {
        Ok(res) => {
            return path_exists(&res);
        }
        Err(e) => {
            panic!("Set QUANTIZED_MODEL_DIR environment variable: {e}")
        }
    }
}

pub fn safetensor_model_path() -> PathBuf {
    match env::var("SAFETENSOR_MODEL_DIR") {
        Ok(res) => {
            return path_exists(&res);
        }
        Err(e) => {
            panic!("Set SAFETENSOR_MODEL_DIR environment variable: {e}")
        }
    }
}

pub fn embedding_model_path() -> PathBuf {
    match env::var("EMBEDDING_MODEL_PATH") {
        Ok(res) => {
            return path_exists(&res);
        }
        Err(e) => {
            panic!("Set EMBEDDING_MODEL_PATH environment variable: {e}")
        }
    }
}

pub fn safetensor_embedding_model_path() -> PathBuf {
    match env::var("ST_EMBEDDING_MODEL_PATH") {
        Ok(res) => return path_exists(&res),
        Err(e) => {
            panic!("Set ST_EMBEDDING_MODEL_PATH environment variable: {e}")
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

pub fn get_device() -> Device {
    if let Ok(cuda) = Device::cuda_if_available(0) {
        return cuda;
    }

    return Device::Cpu;
}
