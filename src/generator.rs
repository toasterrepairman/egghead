use std::error::Error;
use std::time::Duration;
use reqwest::blocking::{Client, Response};
use rust_bert::gpt_neo::{
    GptNeoConfigResources, GptNeoMergesResources, GptNeoModelResources, GptNeoVocabResources,
};
use rust_bert::pipelines::common::ModelType;
use rust_bert::pipelines::sentiment::{Sentiment, SentimentModel};
use rust_bert::pipelines::sequence_classification::{SequenceClassificationConfig, SequenceClassificationModel};
use rust_bert::pipelines::text_generation::{TextGenerationConfig, TextGenerationModel};
use rust_bert::reformer::{ReformerConfigResources, ReformerModelResources, ReformerVocabResources};
use rust_bert::resources::RemoteResource;
use rust_bert::roberta::{RobertaConfigResources, RobertaMergesResources, RobertaModelResources, RobertaVocabResources};
use serenity::json::Value;
use serenity::model::channel::Message;
use tch::Device;

pub fn ask(question: &str, context: &str) -> String {
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
        merges_resource: merges_resource,
        min_length: 20,
        max_length: 150,
        early_stopping: true,
        do_sample: false,
        num_beams: 1,
        num_return_sequences: 1,
        repetition_penalty: 104.5,
        temperature: 3.4,
        diversity_penalty: Some(3.0),
        no_repeat_ngram_size: 3,
        device: Device::Cpu,
        ..Default::default()
    };

    let mut model = TextGenerationModel::new(generate_config)
        .expect("This regularly blows up");
    model.set_device(Device::cuda_if_available());

    let input_context_1 = format!("{}\n", question);
    let mut output = model.generate((&[&input_context_1]), None).pop();
    return output.unwrap().split_off(input_context_1.len())
}

pub fn wiki(question: &str, context: &str) -> String {
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
        merges_resource: merges_resource,
        min_length: 20,
        max_length: 180,
        early_stopping: true,
        do_sample: false,
        num_beams: 1,
        num_return_sequences: 1,
        repetition_penalty: 104.5,
        temperature: 3.4,
        diversity_penalty: Some(3.0),
        no_repeat_ngram_size: 3,
        device: Device::Cpu,
        ..Default::default()
    };

    let mut model = TextGenerationModel::new(generate_config)
        .expect("This regularly blows up");
    model.set_device(Device::cuda_if_available());

    let input_context_1 = format!("{}", question);
    let mut output = model.generate((&[&input_context_1]), None).pop();
    return output.unwrap()
}

pub fn hn(question: &str, context: &str) -> String {
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
        merges_resource: merges_resource,
        min_length: 20,
        max_length: 80,
        early_stopping: true,
        do_sample: false,
        num_beams: 1,
        num_return_sequences: 1,
        repetition_penalty: 104.5,
        temperature: 2.4,
        diversity_penalty: Some(9.0),
        no_repeat_ngram_size: 3,
        device: Device::Cpu,
        ..Default::default()
    };

    let mut model = TextGenerationModel::new(generate_config)
        .expect("This regularly blows up");
    model.set_device(Device::cuda_if_available());

    let input_context_1 = format!("{}", question);
    let mut output = model.generate((&[&input_context_1]), None).pop();
    return output.unwrap()
}

pub fn script(task: &str, context: &str) -> String {
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
        merges_resource: merges_resource,
        min_length: 20,
        max_length: 150,
        early_stopping: true,
        do_sample: false,
        num_beams: 1,
        num_return_sequences: 1,
        repetition_penalty: 150.5,
        temperature: 2.1,
        diversity_penalty: Some(15.0),
        no_repeat_ngram_size: 1,
        device: Device::Cpu,
        ..Default::default()
    };

    let mut model = TextGenerationModel::new(generate_config)
        .expect("This regularly blows up");
    model.set_device(Device::cuda_if_available());

    let input_context_1 = format!("{}", task);
    let mut output = model.generate((&[&input_context_1]), None).pop();
    return output.unwrap()
}

pub fn analyze(context: &str) -> Vec<Sentiment> {
    //    Set-up classifier
    let sentiment_classifier = SentimentModel::new(Default::default()).unwrap();

    //    Define input
    let input = [
        context
    ];

    //    Run model
    return sentiment_classifier.predict(input);
}

pub fn get_chat_response(prompt: &str) -> Result<String, reqwest::Error> {
    let client = Client::new();

    // Make initial GET request to get chat UUID
    let chat_url = "http://localhost:8008/api/chat?model=ggml-alpaca-7B-q4_0.bin&temperature=0.1&top_k=50&top_p=0.95&max_length=1024&context_window=512&repeat_last_n=64&repeat_penalty=1.3&init_prompt=Below%20is%20an%20instruction%20that%20describes%20a%20task.%20Write%20a%20response%20that%20appropriately%20completes%20the%20request.%20The%20response%20must%20be%20accurate%2C%20concise%20and%20evidence-based%20whenever%20possible.%20A%20complete%20answer%20is%20always%20ended%20by%20%5Bend%20of%20text%5D.&n_threads=2";
    let chat_uuid = client.get(chat_url).send()?.text()?;

    // Make POST request to the question endpoint with the prompt
    let question_url = format!("http://localhost:8008/api/{}/question", chat_uuid);
    let response = client.post(&question_url)
        .body(prompt.to_owned())
        .send()?
        .text()?;

    Ok(response)
}
