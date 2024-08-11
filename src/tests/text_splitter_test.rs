#[cfg(test)]
mod text_splitter_text {
    use crate::text_splitter::TextSplitter;
    #[test]
    fn test_split_20_5() {
        let some_long_text: String = "A story about a boy".to_string();

        let text_slitter = TextSplitter::new();

        let splitted_text = text_slitter
            .with_chunk_size(20)
            .with_chunk_overlap(5)
            .split(&some_long_text);

        assert_eq!(splitted_text, vec!["A story about a boy",]);
    }
    #[test]
    fn test_split_5_1() {
        let some_long_text: String = "I know that I know nothing, that say something about the author you know nothing, The guy who said this was a very knowledgeable person and his name was Socrates a great philosopher and Plato's teacher".to_string();

        let text_slitter = TextSplitter::new();

        assert_eq!(
            text_slitter
                .with_chunk_size(25)
                .with_chunk_overlap(3)
                .split(&some_long_text),
            vec![
                "I know that I know",
                "know nothing, that say",
                "say something about the",
                "the author you know",
                "know nothing, The guy",
                "guy who said this was a",
                "was a very knowledgeable",
                "knowledgeable person and",
                "and his name was",
                "was Socrates a great",
                "great philosopher and",
                "and Plato's teacher"
            ]
        );
    }

    #[test]
    fn test_split_not_equal() {
        let some_long_text: String = "I know that I know nothing, that say something about the author you know nothing, The guy who said this was a very knowledgeable person and his name was Socrates a great philosopher and Plato's teacher".to_string();

        let text_slitter = TextSplitter::new();

        assert_ne!(
            text_slitter
                .with_chunk_size(25)
                .with_chunk_overlap(3)
                .split(&some_long_text),
            vec![
                "I know that I know",
                "know nothing, that say",
                "say something about the",
                "the author you know",
                "know nothing, The guy",
                "guy who said this was a",
                "was a very knowledgeable",
                "knowledgeable person and",
                "and his name was",
                "was Socrates a great",
                "great philosopher and",
            ]
        );
    }

    #[test]
    fn test_split_no_infinite_loop() {
        let some_long_text: String = "I have nothing to say here.".to_string();

        let text_slitter = TextSplitter::new();

        assert!(!text_slitter
            .with_chunk_size(150)
            .with_chunk_overlap(5)
            .split(&some_long_text)
            .is_empty());
    }
}
