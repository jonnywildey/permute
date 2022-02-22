use crate::process::PermuteNodeName;

pub fn get_processor_display_name(name: PermuteNodeName) -> String {
    match name {
        PermuteNodeName::Reverse => String::from("Reverse"),
        PermuteNodeName::Chorus => String::from("Chorus"),
        PermuteNodeName::Phaser => String::from("Phaser"),
        PermuteNodeName::DoubleSpeed => String::from("Double speed"),
        PermuteNodeName::HalfSpeed => String::from("Half speed"),
        PermuteNodeName::RandomPitch => String::from("Random pitch"),
        PermuteNodeName::GranularTimeStretch => String::from("Granular stretch"),
        PermuteNodeName::Reverb => String::from("Reverb"),
        PermuteNodeName::Fuzz => String::from("Fuzz"),
        PermuteNodeName::Saturate => String::from("Saturate"),
        PermuteNodeName::Flutter => String::from("Flutter"),
        PermuteNodeName::Flange => String::from("Flange"),
        PermuteNodeName::MetallicDelay => String::from("Metallic delay"),
        PermuteNodeName::RhythmicDelay => String::from("Rhythmic delay"),
        PermuteNodeName::Wow => String::from("Wow"),
        PermuteNodeName::Normalise => String::from("Normalise"),
        PermuteNodeName::Trim => String::from("Trim"),
        PermuteNodeName::SampleRateConversionHigh => String::from("Sample rate conversion high"),
        PermuteNodeName::SampleRateConversionOriginal => String::from("Sample rate conversion low"),
    }
}

pub fn get_processor_from_display_name(name: &str) -> Result<PermuteNodeName, String> {
    match name {
        "Reverse" => Ok(PermuteNodeName::Reverse),
        "Chorus" => Ok(PermuteNodeName::Chorus),
        "Phaser" => Ok(PermuteNodeName::Phaser),
        "Double speed" => Ok(PermuteNodeName::DoubleSpeed),
        "Flutter" => Ok(PermuteNodeName::Flutter),
        "Flange" => Ok(PermuteNodeName::Flange),
        "Half speed" => Ok(PermuteNodeName::HalfSpeed),
        "Random pitch" => Ok(PermuteNodeName::RandomPitch),
        "Metallic delay" => Ok(PermuteNodeName::MetallicDelay),
        "Rhythmic delay" => Ok(PermuteNodeName::RhythmicDelay),
        "Granular stretch" => Ok(PermuteNodeName::GranularTimeStretch),
        "Reverb" => Ok(PermuteNodeName::Reverb),
        "Fuzz" => Ok(PermuteNodeName::Fuzz),
        "Saturate" => Ok(PermuteNodeName::Saturate),
        "Wow" => Ok(PermuteNodeName::Wow),
        "Normalise" => Ok(PermuteNodeName::Normalise),
        "Trim" => Ok(PermuteNodeName::Trim),
        "Sample rate conversion high" => Ok(PermuteNodeName::SampleRateConversionHigh),
        "Sample rate conversion low" => Ok(PermuteNodeName::SampleRateConversionOriginal),
        _ => Err(format!("{} not found", name)),
    }
}
