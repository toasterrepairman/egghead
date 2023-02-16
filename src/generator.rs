use rust_bert::gpt_neo::{
    GptNeoConfigResources, GptNeoMergesResources, GptNeoModelResources, GptNeoVocabResources,
};
use rust_bert::pipelines::common::ModelType;
use rust_bert::pipelines::sentiment::SentimentModel;
use rust_bert::pipelines::sequence_classification::{SequenceClassificationConfig, SequenceClassificationModel};
use rust_bert::pipelines::text_generation::{TextGenerationConfig, TextGenerationModel};
use rust_bert::reformer::{ReformerConfigResources, ReformerModelResources, ReformerVocabResources};
use rust_bert::resources::RemoteResource;
use rust_bert::roberta::{RobertaConfigResources, RobertaMergesResources, RobertaModelResources, RobertaVocabResources};
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
        max_length: 180,
        early_stopping: true,
        do_sample: false,
        num_beams: 1,
        num_return_sequences: 1,
        repetition_penalty: 105.5,
        temperature: 3.2,
        diversity_penalty: Some(15.0),
        no_repeat_ngram_size: 3,
        device: Device::Cpu,
        ..Default::default()
    };

    let mut model = TextGenerationModel::new(generate_config)
        .expect("This regularly blows up");
    model.set_device(Device::cuda_if_available());

    let input_context_1 = format!("Q: {}\nA:", question);
    let mut output = model.generate((&[input_context_1]), None).pop();
    return output.unwrap()
}

pub fn analyze(context: String) {
    //    Set-up classifier
    let sentiment_classifier = SentimentModel::new(Default::default()).unwrap();

    //    Define input
    let input = [

    ];

    //    Run model
    let output = sentiment_classifier.predict(input);
}