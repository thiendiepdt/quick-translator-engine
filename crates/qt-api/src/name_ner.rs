//! Optional token-classification ONNX adapter.

use std::sync::Arc;

use qt_core::{CharRange, NameEntityType};

#[derive(Debug, Clone)]
pub struct NerNameSpan {
    pub text: String,
    pub entity_type: NameEntityType,
    pub score: f32,
    pub range: CharRange,
}

pub trait NameEntityRecognizer: Send + Sync {
    fn recognize(&self, text: &str) -> Result<Vec<NerNameSpan>, String>;
}

pub fn from_env() -> Result<Option<Arc<dyn NameEntityRecognizer>>, String> {
    #[cfg(feature = "onnx")]
    {
        let Some(model) = non_empty_env("QT_NER_MODEL") else {
            return Ok(None);
        };
        let tokenizer = non_empty_env("QT_NER_TOKENIZER")
            .ok_or_else(|| "QT_NER_MODEL is set but QT_NER_TOKENIZER is missing".to_string())?;
        let config = non_empty_env("QT_NER_CONFIG")
            .ok_or_else(|| "QT_NER_MODEL is set but QT_NER_CONFIG is missing".to_string())?;
        OnnxNameRecognizer::load(&model, &tokenizer, &config)
            .map(|recognizer| Some(Arc::new(recognizer) as Arc<dyn NameEntityRecognizer>))
    }
    #[cfg(not(feature = "onnx"))]
    {
        if non_empty_env("QT_NER_MODEL").is_some() {
            return Err(
                "QT_NER_MODEL is set but qt-api was built without the `onnx` feature".to_string(),
            );
        }
        Ok(None)
    }
}

fn non_empty_env(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[cfg(feature = "onnx")]
mod onnx {
    use std::collections::HashMap;
    use std::sync::Mutex;

    use ort::session::Session;
    use ort::value::Tensor;
    use serde::Deserialize;
    use tokenizers::Tokenizer;

    use super::{NameEntityRecognizer, NerNameSpan};
    use qt_core::{CharRange, NameEntityType};

    const CHUNK_CHARACTERS: usize = 400;
    const CHUNK_OVERLAP: usize = 32;

    pub struct OnnxNameRecognizer {
        tokenizer: Tokenizer,
        session: Mutex<Session>,
        labels: Vec<String>,
    }

    #[derive(Deserialize)]
    struct ModelConfig {
        id2label: HashMap<String, String>,
    }

    impl OnnxNameRecognizer {
        pub fn load(model: &str, tokenizer: &str, config: &str) -> Result<Self, String> {
            let tokenizer = Tokenizer::from_file(tokenizer)
                .map_err(|error| format!("failed to load NER tokenizer: {error}"))?;
            let config: ModelConfig = serde_json::from_slice(
                &std::fs::read(config)
                    .map_err(|error| format!("failed to read NER config: {error}"))?,
            )
            .map_err(|error| format!("failed to parse NER config: {error}"))?;
            let mut labels: Vec<(usize, String)> = config
                .id2label
                .into_iter()
                .map(|(id, label)| {
                    id.parse::<usize>()
                        .map(|id| (id, label))
                        .map_err(|_| format!("invalid NER label id: {id}"))
                })
                .collect::<Result<_, _>>()?;
            labels.sort_by_key(|(id, _)| *id);
            let labels = labels.into_iter().map(|(_, label)| label).collect();
            let builder = Session::builder()
                .map_err(|error| format!("failed to initialize ONNX Runtime: {error}"))?;
            let mut builder = builder
                .with_intra_threads(1)
                .map_err(|error| format!("failed to configure NER session: {error}"))?;
            let session = builder
                .commit_from_file(model)
                .map_err(|error| format!("failed to load NER ONNX model: {error}"))?;
            Ok(Self {
                tokenizer,
                session: Mutex::new(session),
                labels,
            })
        }

        fn recognize_chunk(
            &self,
            chapter: &str,
            chunk: &str,
            chunk_byte_start: usize,
        ) -> Result<Vec<NerNameSpan>, String> {
            let encoding = self
                .tokenizer
                .encode(chunk, true)
                .map_err(|error| format!("NER tokenization failed: {error}"))?;
            let length = encoding.len();
            if length == 0 {
                return Ok(Vec::new());
            }
            let ids: Vec<i64> = encoding
                .get_ids()
                .iter()
                .map(|value| *value as i64)
                .collect();
            let mask: Vec<i64> = encoding
                .get_attention_mask()
                .iter()
                .map(|value| *value as i64)
                .collect();
            let type_ids: Vec<i64> = encoding
                .get_type_ids()
                .iter()
                .map(|value| *value as i64)
                .collect();
            let mut session = self
                .session
                .lock()
                .map_err(|_| "NER session lock is poisoned".to_string())?;
            let input_names: Vec<String> = session
                .inputs()
                .iter()
                .map(|input| input.name().to_string())
                .collect();
            let mut inputs = Vec::with_capacity(input_names.len());
            for name in input_names {
                let values = match name.as_str() {
                    "input_ids" => ids.clone(),
                    "attention_mask" => mask.clone(),
                    "token_type_ids" => type_ids.clone(),
                    other => return Err(format!("unsupported NER model input: {other}")),
                };
                let tensor = Tensor::from_array(([1, length], values))
                    .map_err(|error| format!("failed to build NER input: {error}"))?;
                inputs.push((name, tensor));
            }
            let outputs = session
                .run(inputs)
                .map_err(|error| format!("NER inference failed: {error}"))?;
            let output = outputs
                .iter()
                .next()
                .map(|(_, output)| output)
                .ok_or_else(|| "NER model returned no outputs".to_string())?;
            let (shape, logits) = output
                .try_extract_tensor::<f32>()
                .map_err(|error| format!("invalid NER logits: {error}"))?;
            if shape.len() != 3 || shape[1] as usize != length {
                return Err(format!("unexpected NER output shape: {shape:?}"));
            }
            let label_count = shape[2] as usize;
            if label_count != self.labels.len() {
                return Err(format!(
                    "NER model has {label_count} labels but config has {}",
                    self.labels.len()
                ));
            }
            decode_bio(
                chapter,
                chunk,
                chunk_byte_start,
                encoding.get_offsets(),
                logits,
                label_count,
                &self.labels,
            )
        }
    }

    impl NameEntityRecognizer for OnnxNameRecognizer {
        fn recognize(&self, text: &str) -> Result<Vec<NerNameSpan>, String> {
            let mut spans = Vec::new();
            for (byte_start, chunk) in char_chunks(text, CHUNK_CHARACTERS, CHUNK_OVERLAP) {
                spans.extend(self.recognize_chunk(text, chunk, byte_start)?);
            }
            spans.sort_by_key(|span| span.range.start);
            spans.dedup_by(|left, right| {
                left.range == right.range && left.entity_type == right.entity_type
            });
            Ok(spans)
        }
    }

    fn decode_bio(
        chapter: &str,
        chunk: &str,
        chunk_byte_start: usize,
        offsets: &[(usize, usize)],
        logits: &[f32],
        label_count: usize,
        labels: &[String],
    ) -> Result<Vec<NerNameSpan>, String> {
        let mut spans = Vec::new();
        let mut current: Option<(usize, usize, NameEntityType, f32, usize)> = None;
        for (index, &(start, end)) in offsets.iter().enumerate() {
            if start == end {
                continue;
            }
            let row = &logits[index * label_count..(index + 1) * label_count];
            let (label_index, confidence) = argmax_confidence(row);
            let label = &labels[label_index];
            let Some((prefix, entity_type)) = parse_bio_label(label) else {
                flush_span(chapter, chunk_byte_start, &mut current, &mut spans)?;
                continue;
            };
            let continues = prefix == 'I'
                && current
                    .as_ref()
                    .is_some_and(|(_, previous_end, kind, _, _)| {
                        *kind == entity_type && *previous_end == start
                    });
            if continues {
                if let Some((_, current_end, _, score_sum, count)) = &mut current {
                    *current_end = end;
                    *score_sum += confidence;
                    *count += 1;
                }
            } else {
                flush_span(chapter, chunk_byte_start, &mut current, &mut spans)?;
                current = Some((start, end, entity_type, confidence, 1));
            }
        }
        flush_span(chapter, chunk_byte_start, &mut current, &mut spans)?;
        let _ = chunk;
        Ok(spans)
    }

    fn flush_span(
        chapter: &str,
        chunk_byte_start: usize,
        current: &mut Option<(usize, usize, NameEntityType, f32, usize)>,
        spans: &mut Vec<NerNameSpan>,
    ) -> Result<(), String> {
        let Some((start, end, entity_type, score_sum, count)) = current.take() else {
            return Ok(());
        };
        let absolute_start = chunk_byte_start + start;
        let absolute_end = chunk_byte_start + end;
        let Some(text) = chapter.get(absolute_start..absolute_end) else {
            return Err("NER tokenizer returned an invalid UTF-8 offset".to_string());
        };
        if text.chars().count() < 2 {
            return Ok(());
        }
        spans.push(NerNameSpan {
            text: text.to_string(),
            entity_type,
            score: (score_sum / count as f32).clamp(0.0, 1.0),
            range: CharRange {
                start: chapter[..absolute_start].encode_utf16().count(),
                length: text.encode_utf16().count(),
            },
        });
        Ok(())
    }

    fn parse_bio_label(label: &str) -> Option<(char, NameEntityType)> {
        let upper = label.to_ascii_uppercase();
        let prefix = upper.chars().next()?;
        if !matches!(prefix, 'B' | 'I') {
            return None;
        }
        let kind = if upper.contains("PER") || upper.contains("PERSON") || upper.contains("NAME") {
            NameEntityType::Person
        } else if upper.contains("LOC") || upper.contains("PLACE") {
            NameEntityType::Location
        } else if upper.contains("ORG") {
            NameEntityType::Organization
        } else {
            return None;
        };
        Some((prefix, kind))
    }

    fn argmax_confidence(values: &[f32]) -> (usize, f32) {
        let (index, max) = values
            .iter()
            .copied()
            .enumerate()
            .max_by(|(_, left), (_, right)| left.total_cmp(right))
            .unwrap_or((0, 0.0));
        let denominator: f32 = values.iter().map(|value| (*value - max).exp()).sum();
        (
            index,
            if denominator > 0.0 {
                1.0 / denominator
            } else {
                0.0
            },
        )
    }

    fn char_chunks(text: &str, max_chars: usize, overlap: usize) -> Vec<(usize, &str)> {
        let max_chars = max_chars.max(1);
        let step = max_chars.saturating_sub(overlap).max(1);
        let mut boundaries: Vec<usize> = text.char_indices().map(|(index, _)| index).collect();
        boundaries.push(text.len());
        let mut result = Vec::new();
        let character_count = boundaries.len().saturating_sub(1);
        let mut start_character = 0;
        while start_character < character_count {
            let end_character = (start_character + max_chars).min(character_count);
            let byte_start = boundaries[start_character];
            let byte_end = boundaries[end_character];
            result.push((byte_start, &text[byte_start..byte_end]));
            if end_character == character_count {
                break;
            }
            start_character += step;
        }
        result
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn maps_supported_bio_labels() {
            assert_eq!(
                parse_bio_label("B-PER"),
                Some(('B', NameEntityType::Person))
            );
            assert_eq!(
                parse_bio_label("I-LOC"),
                Some(('I', NameEntityType::Location))
            );
            assert_eq!(parse_bio_label("O"), None);
        }

        #[test]
        fn chunks_on_utf8_boundaries() {
            assert_eq!(
                char_chunks("一二三四五", 2, 1),
                vec![(0, "一二"), (3, "二三"), (6, "三四"), (9, "四五")]
            );
        }
    }
}

#[cfg(feature = "onnx")]
use onnx::OnnxNameRecognizer;
