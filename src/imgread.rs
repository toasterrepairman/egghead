use std::path::Path;

use anyhow::Error as E;

use candle::{Device, Result, Tensor};

use candle_examples::token_output_stream::TokenOutputStream;
use candle_transformers::models::blip;
use candle_transformers::models::quantized_blip;

use tokenizers::Tokenizer;

enum Model {
    M(blip::BlipForConditionalGeneration),
    Q(quantized_blip::BlipForConditionalGeneration),
}

const SEP_TOKEN_ID: u32 = 102;

impl Model {
    fn text_decoder_forward(&mut self, xs: &Tensor, img_xs: &Tensor) -> Result<Tensor> {
        match self {
            Self::M(m) => m.text_decoder().forward(xs, img_xs),
            Self::Q(m) => m.text_decoder().forward(xs, img_xs),
        }
    }
}

pub fn load_image<P: AsRef<std::path::Path>>(p: P) -> Result<Tensor> {
    let img = image::io::Reader::open(p)?
        .decode()
        .map_err(candle::Error::wrap)?
        .resize_to_fill(384, 384, image::imageops::FilterType::Triangle);
    let img = img.to_rgb8();
    let data = img.into_raw();
    let data = Tensor::from_vec(data, (384, 384, 3), &Device::Cpu)?.permute((2, 0, 1))?;
    let mean =
        Tensor::new(&[0.48145466f32, 0.4578275, 0.40821073], &Device::Cpu)?.reshape((3, 1, 1))?;
    let std = Tensor::new(&[0.26862954f32, 0.261_302_6, 0.275_777_1], &Device::Cpu)?
        .reshape((3, 1, 1))?;
    (data.to_dtype(candle::DType::F32)? / 255.)?
        .broadcast_sub(&mean)?
        .broadcast_div(&std)
}

pub fn img_react(path: &Path) -> anyhow::Result<String> {
    let model_file = {
        let api = hf_hub::api::sync::Api::new()?;
        let api = api.model("lmz/candle-blip".to_string());
        api.get("blip-image-captioning-large-q4k.gguf")?
    };
    let tokenizer = {
        let api = hf_hub::api::sync::Api::new()?;
        let api = api.model("lmz/candle-blip".to_string());
        api.get("tokenizer.json")?
    };
    let tokenizer = Tokenizer::from_file(tokenizer).map_err(E::msg)?;
    let mut tokenizer = TokenOutputStream::new(tokenizer);
    let mut logits_processor =
        candle_transformers::generation::LogitsProcessor::new(1337, None, None);

    let config = blip::Config::image_captioning_large();

    let (image_embeds, device, mut model) = {
        let device = Device::Cpu;

        let image = load_image(path)?.to_device(&device)?;
        println!("loaded image {image:?}");

        let vb = quantized_blip::VarBuilder::from_gguf(model_file)?;
        let model = quantized_blip::BlipForConditionalGeneration::new(&config, vb)?;
        let image_embeds = image.unsqueeze(0)?.apply(model.vision_model())?;
        (image_embeds, device, Model::Q(model))
    };

    let mut token_ids = vec![30522u32];
    let mut result = String::new();

    for index in 0..1000 {
        let context_size = if index > 0 { 1 } else { token_ids.len() };
        let start_pos = token_ids.len().saturating_sub(context_size);
        let input_ids = Tensor::new(&token_ids[start_pos..], &device)?.unsqueeze(0)?;
        let logits = model.text_decoder_forward(&input_ids, &image_embeds)?;
        let logits = logits.squeeze(0)?;
        let logits = logits.get(logits.dim(0)? - 1)?;
        let token = logits_processor.sample(&logits)?;
        if token == SEP_TOKEN_ID {
            break;
        }
        token_ids.push(token);

        if let Some(t) = tokenizer.next_token(token)? {
            result.push_str(&format!("{}", t));
        }

        println!("{}", result);
    }

    if let Some(rest) = tokenizer.decode_rest().map_err(E::msg)? {
        print!("{rest}");
    }

    Ok(result.to_string())
}
