extern crate anyhow;

use rust_bert::pipelines::common::ModelType;
use rust_bert::pipelines::summarization::{SummarizationConfig, SummarizationModel};
use rust_bert::resources::RemoteResource;
use rust_bert::t5::{T5ConfigResources, T5ModelResources, T5VocabResources};

pub fn get_chat_response(temp: &str, init: &str, prompt: &str) -> Result<String, reqwest::Error> {
    let config_resource = RemoteResource::from_pretrained(T5ConfigResources::T5_BASE);
    let vocab_resource = RemoteResource::from_pretrained(T5VocabResources::T5_BASE);
    let weights_resource = RemoteResource::from_pretrained(T5ModelResources::T5_BASE);

    let generation_config = TextGenerationConfig {
        model_type: ModelType::T5,
        model_resource,
        config_resource,
        vocab_resource,
        merges_resource: Some(merges_resource),
        min_length: 10,
        max_length: Some(32),
        do_sample: false,
        early_stopping: true,
        num_beams: 1,
        num_return_sequences: 1,
        device: Device::cuda_if_available(),
        ..Default::default()
    };

    let model = TextGenerationModel::new(generation_config)?;

    // Generate text

    let prompts = [
        init,
        prompt,
    ];
    let output = model.generate(&prompts, None);

    Ok(answer)
}