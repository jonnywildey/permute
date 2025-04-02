// External dependencies
use rand::{thread_rng, Rng};

// Internal modules
use crate::{
    processors::filter::{FilterParams, OscillatingFilterParams, LineFilterParams, FilterForm, filter, oscillating_filter, multi_line_filter},
    random_processors::utils::{format_float, format_hz, format_float_percent},
    process::{ProcessorParams, PermuteNodeName, ProcessorAttribute, PermuteNodeEvent},
    permute_files::PermuteUpdate,
    random_process::{start_event, complete_event},
    permute_error::PermuteError,
};

pub fn random_filter(params: &mut ProcessorParams) -> Result<ProcessorParams, PermuteError> {
    start_event!(PermuteNodeName::Filter, params);
    let mut rng = thread_rng();

    let freqs = [
        200.0, 250.0, 300.0, 400.0, 500.0, 600.0, 800.0, 1000.0, 1200.0, 1600.0, 2000.0, 2400.0,
        3200.0, 4000.0, 4800.0, 6400.0,
    ];
    let types = [
        biquad::Type::HighPass,
        biquad::Type::LowPass,
        biquad::Type::BandPass,
    ];

    let filter_type = types[rng.gen_range(0..types.len())];
    let frequency = freqs[rng.gen_range(0..freqs.len())];
    let q = rng.gen_range(0.15_f64..1.2_f64);
    let form = FilterForm::Form2;

    // Update processor attributes before processing
    params.update_processor_attributes(
        params.permutation.clone(),
        vec![
            ProcessorAttribute {
                key: "Filter Type".to_string(),
                value: format!("{:?}", filter_type),
            },
            ProcessorAttribute {
                key: "Frequency".to_string(),
                value: format_hz(frequency),
            },
            ProcessorAttribute {
                key: "Q".to_string(),
                value: format_float(q),
            },
            ProcessorAttribute {
                key: "Form".to_string(),
                value: format!("{:?}", form),
            },
        ],
    );

    let filter_params = FilterParams {
        filter_type,
        frequency,
        q: Some(q),
        form: form.clone(),
    };

    let new_params = filter(params, &filter_params)?;
    complete_event!(PermuteNodeName::Filter, new_params);
    Ok(new_params)
}

pub fn random_oscillating_filter(
    params: &mut ProcessorParams,
) -> Result<ProcessorParams, PermuteError> {
    start_event!(PermuteNodeName::OscillatingFilter, params);

    let mut rng = thread_rng();

    let freqs = [
        200.0, 250.0, 300.0, 400.0, 500.0, 600.0, 800.0, 1000.0, 1200.0, 1600.0, 2000.0, 2400.0,
        3200.0, 4000.0, 4800.0, 6400.0,
    ];
    let lfo_rates = [
        0.2, 0.3, 0.4, 0.5, 0.75, 1.0, 1.2, 1.4, 1.6, 1.8, 2.0, 3.0, 4.0, 6.0, 8.0, 10.0, 15.0,
        25.0, 45.0, 80.0,
    ];
    let lfo_factors = [0.5, 0.6, 0.7, 0.8, 0.9, 0.95];
    let types = [
        biquad::Type::HighPass,
        biquad::Type::LowPass,
        biquad::Type::BandPass,
    ];

    let filter_type = types[rng.gen_range(0..types.len())];
    let frequency = freqs[rng.gen_range(0..freqs.len())];
    let lfo_rate = lfo_rates[rng.gen_range(0..lfo_rates.len())];
    let lfo_factor = lfo_factors[rng.gen_range(0..lfo_factors.len())];
    let q = rng.gen_range(0.5_f64..1.3_f64);
    let form = FilterForm::Form2;

    // Update processor attributes before processing
    params.update_processor_attributes(
        params.permutation.clone(),
        vec![
            ProcessorAttribute {
                key: "Filter Type".to_string(),
                value: format!("{:?}", filter_type),
            },
            ProcessorAttribute {
                key: "Frequency".to_string(),
                value: format_hz(frequency),
            },
            ProcessorAttribute {
                key: "LFO Rate".to_string(),
                value: format_hz(lfo_rate),
            },
            ProcessorAttribute {
                key: "LFO Factor".to_string(),
                value: format_float_percent(lfo_factor),
            },
            ProcessorAttribute {
                key: "Q".to_string(),
                value: format_float(q),
            },
            ProcessorAttribute {
                key: "Form".to_string(),
                value: format!("{:?}", form),
            },
        ],
    );

    let filter_params = OscillatingFilterParams {
        filter_type,
        frequency,
        q: Some(q),
        form: form.clone(),
        lfo_rate,
        lfo_factor,
    };

    let new_params = oscillating_filter(params, &filter_params)?;
    complete_event!(PermuteNodeName::OscillatingFilter, new_params);
    Ok(new_params)
}

pub fn random_line_filter(params: &mut ProcessorParams) -> Result<ProcessorParams, PermuteError> {
    start_event!(PermuteNodeName::LineFilter, params);

    let mut rng = thread_rng();

    let freqs = [
        100.0, 150.0, 160.0, 175.0, 200.0, 220.0, 250.0, 300.0, 350.0, 400.0, 450.0, 500.0, 550.0,
        600.0, 650.0, 800.0, 850.0, 1000.0, 1050.0, 1200.0, 1250.0, 1600.0, 2000.0, 2400.0, 3200.0,
        3500.0, 4000.0, 4800.0, 5200.0, 6400.0, 8000.0, 8500.0, 10000.0, 12000.0, 13000.0, 14000.0,
        15000.0,
    ];
    let types = [
        biquad::Type::HighPass,
        biquad::Type::LowPass,
        biquad::Type::LowPass, // make low pass most likely
        biquad::Type::BandPass,
    ];

    let filter_type = types[rng.gen_range(0..types.len())];
    let hz_from = freqs[rng.gen_range(0..freqs.len())];
    let hz_to = freqs[rng.gen_range(0..freqs.len())];
    let q = rng.gen_range(0.5_f64..1.35_f64);
    let form = FilterForm::Form2;

    // Update processor attributes before processing
    params.update_processor_attributes(
        params.permutation.clone(),
        vec![
            ProcessorAttribute {
                key: "Filter Type".to_string(),
                value: format!("{:?}", filter_type),
            },
            ProcessorAttribute {
                key: "From".to_string(),
                value: format_hz(hz_from),
            },
            ProcessorAttribute {
                key: "To".to_string(),
                value: format_hz(hz_to),
            },
            ProcessorAttribute {
                key: "Q".to_string(),
                value: format_float(q),
            },
            ProcessorAttribute {
                key: "Form".to_string(),
                value: format!("{:?}", form),
            },
        ],
    );

    let filter_params = LineFilterParams {
        filter_type,
        form: form.clone(),
        hz_from,
        hz_to,
        q: Some(q),
    };

    let new_params = multi_line_filter(&params.to_owned(), &filter_params)?;
    complete_event!(PermuteNodeName::LineFilter, new_params);
    Ok(new_params)
}