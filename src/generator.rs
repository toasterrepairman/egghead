use rust_bert::gpt_neo::{
    GptNeoConfigResources, GptNeoMergesResources, GptNeoModelResources, GptNeoVocabResources,
};
use rust_bert::pipelines::common::ModelType;
use rust_bert::pipelines::text_generation::{TextGenerationConfig, TextGenerationModel};
use rust_bert::resources::RemoteResource;
use tch::Device;

pub fn generate(prompt: &str, min_len: i64, max_len: Option<i64>) -> String {
    //    Set-up model resources
    let config_resource = Box::new(RemoteResource::from_pretrained(
        GptNeoConfigResources::GPT_NEO_125M,
    ));
    let vocab_resource = Box::new(RemoteResource::from_pretrained(
        GptNeoVocabResources::GPT_NEO_125M,
    ));
    let merges_resource = Box::new(RemoteResource::from_pretrained(
        GptNeoMergesResources::GPT_NEO_125M,
    ));
    let model_resource = Box::new(RemoteResource::from_pretrained(
        GptNeoModelResources::GPT_NEO_125M,
    ));
    let generate_config = TextGenerationConfig {
        model_type: ModelType::GPTNeo,
        model_resource,
        config_resource,
        vocab_resource,
        merges_resource: Some(merges_resource),
        min_length: min_len,
        max_length: max_len,
        do_sample: true,
        early_stopping: true,
        num_beams: 3,
        num_beam_groups: Some(3),
        num_return_sequences: 1,
        device: Device::Cpu,
        repetition_penalty: 10.0,
        temperature: 4.0,
        top_k: 40,
        length_penalty: 3.0,
        diversity_penalty: Some(3.0),
        ..Default::default()
    };

    let mut model = TextGenerationModel::new(generate_config)
        .expect("Uh oh, it's the generator broken.");
    model.set_device(Device::cuda_if_available());

    let input_context_1 = prompt;
    let output = model.generate(&[input_context_1], None);

    let response: String = output.into_iter().collect();
    return response
}
