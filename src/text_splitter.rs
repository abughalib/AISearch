use std::io::BufRead;
use std::{
    fs::File,
    io::{self, BufReader},
};

pub struct TextSplitter {
    chunk_size: usize,
    chunk_overlap: usize,
    separator: String,
}

impl TextSplitter {
    pub fn new() -> Self {
        Self {
            chunk_size: 1000,
            chunk_overlap: 150,
            separator: "\n\n".to_string(),
        }
    }

    pub fn with_chunk_size(mut self, chunk_size: usize) -> Self {
        if chunk_size < 20 {
            // To avoid infinite loops
            panic!("Chunk size must be at least 100");
        }
        self.chunk_size = chunk_size;
        self
    }

    pub fn with_chunk_overlap(mut self, chunk_overlap: usize) -> Self {

        if chunk_overlap >= self.chunk_size && chunk_overlap < 10 {
            // To avoid infinite loops
            panic!("Chunk overlap must be smaller than chunk size");
        }
        self.chunk_overlap = chunk_overlap;
        self
    }

    pub fn with_separator(mut self, separator: String) -> Self {
        self.separator = separator;
        self
    }

    pub fn split(&self, text: &str) -> Vec<String> {
        let mut chunks = Vec::new();
        let mut start = 0;
    
        // Ensure chunk size is larger than overlap
        if self.chunk_size <= self.chunk_overlap {
            panic!("Chunk size must be larger than chunk overlap");
        }
    
        while start < text.len() {
            // Determine the end of the current chunk
            let mut end = std::cmp::min(start + self.chunk_size, text.len());
    
            // Adjust the end to avoid splitting in the middle of a word
            if end < text.len() {
                if let Some(last_space) = text[start..end].rfind(' ') {
                    end = start + last_space;
                }
            }
    
            // Add the chunk, avoiding unnecessary allocations
            let chunk = &text[start..end];
            let chunk = if start == 0 && end >= text.len() {
                text // If the chunk is the entire text, use the original string
            } else {
                chunk.trim() // Only trim when necessary
            };
            if !chunk.is_empty() {
                chunks.push(chunk.to_string());
            }
    
            // Break if we're at the end of the text
            if end >= text.len() {
                break;
            }
    
            // Move the start index to the next chunk, taking overlap into account
            let new_start = end.saturating_sub(self.chunk_overlap);
    
            // Adjust the start to avoid splitting in the middle of a word
            let mut adjusted_start = new_start;
            if adjusted_start < text.len() && !text.as_bytes()[adjusted_start].is_ascii_whitespace() {
                adjusted_start = self.find_nearest_space_to_left(text, adjusted_start);
            }
    
            // Ensure that the start index always moves forward
            if adjusted_start <= start {
                start = end; // Move start to the end of the last chunk if no forward movement
            } else {
                start = adjusted_start;
            }
    
            // Safety check to avoid infinite loops
            if start >= text.len() {
                break;
            }
        }
    
        chunks
    }    

    fn find_nearest_space_to_left(&self, text: &str, index: usize) -> usize {
        if index == 0 {
            return index;
        }

        let bytes = text.as_bytes();
        for i in (0..=index).rev() {
            if bytes[i].is_ascii_whitespace() {
                return i + 1;
            }
        }

        return index;
    }
    pub fn split_from_file(&self, file_path: &str) -> io::Result<Vec<String>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);

        let mut chunks = Vec::new();
        let mut buffer = String::new();
        let mut start = 0;

        for line in reader.lines() {
            let line = line?;
            buffer.push_str(&line);
            buffer.push('\n');

            while start < buffer.len() {
                // Determine the end of the current chunk
                let mut end = std::cmp::min(start + self.chunk_size, buffer.len());

                // Adjust the end to avoid splitting in the middle of a word
                if end < buffer.len() {
                    if let Some(last_space) = buffer[start..end].rfind(' ') {
                        end = start + last_space;
                    }
                }

                // Add the chunk
                let chunk = &buffer[start..end].trim();
                if !chunk.is_empty() {
                    chunks.push(chunk.to_string());
                }

                // Break if we're at the end of the buffer
                if end >= buffer.len() {
                    break;
                }

                // Move the start index to the next chunk, taking overlap into account
                let new_start = end.saturating_sub(self.chunk_overlap);

                // Adjust the start to avoid splitting in the middle of a word
                let mut adjusted_start = new_start;
                if adjusted_start < buffer.len()
                    && !buffer.as_bytes()[adjusted_start].is_ascii_whitespace()
                {
                    adjusted_start = self.find_nearest_space_to_left(&buffer, adjusted_start);
                }

                // Ensure that the start index always moves forward
                if adjusted_start <= start {
                    start = end; // Move start to the end of the last chunk if no forward movement
                } else {
                    start = adjusted_start;
                }
            }

            // Reset buffer and start position if it has been fully processed
            if start >= buffer.len() {
                buffer.clear();
                start = 0;
            }
        }

        Ok(chunks)
    }
}
