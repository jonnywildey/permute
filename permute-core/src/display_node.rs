use crate::process::PermuteNodeName;

pub fn get_processor_display_name(name: PermuteNodeName) -> String {
    match name {
        PermuteNodeName::Reverse => String::from("Reverse"),
        PermuteNodeName::Chorus => String::from("Chorus"),
        PermuteNodeName::Phaser => String::from("Phaser"),
        PermuteNodeName::DoubleSpeed => String::from("Double speed"),
        PermuteNodeName::Flutter => String::from("Flutter"),
        PermuteNodeName::Flange => String::from("Flange"),
        PermuteNodeName::HalfSpeed => String::from("Half speed"),
        PermuteNodeName::MetallicDelay => String::from("Metallic delay"),
        PermuteNodeName::RhythmicDelay => String::from("Rhythmic delay"),
        PermuteNodeName::Wow => String::from("Wow"),
        PermuteNodeName::Normalise => String::from("Normalise"),
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
        "Metallic delay" => Ok(PermuteNodeName::MetallicDelay),
        "Rhythmic delay" => Ok(PermuteNodeName::RhythmicDelay),
        "Wow" => Ok(PermuteNodeName::Wow),
        "Normalise" => Ok(PermuteNodeName::Normalise),
        "Sample rate conversion high" => Ok(PermuteNodeName::SampleRateConversionHigh),
        "Sample rate conversion low" => Ok(PermuteNodeName::SampleRateConversionOriginal),
        _ => Err(format!("{} not found", name)),
    }
}
