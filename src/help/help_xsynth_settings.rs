macro_rules! help_mod {
    { $($key:ident = $value:expr),+, }=> {
        $(
            macro_rules! $key {
                () => {
                    $value
                };
            }
        )+
    };
}

help_mod! {
    buffer_ms_short_help = "The size of the event buffer",
    buffer_ms_long_help =
        "The amount of time in milliseconds that events \
        are held in the buffer before they are played. Making this \
        option larger can help MIDI files play in real-time that wouldn't \
        otherwise.",

    sfz_path_short_help = "The path to an SFZ SoundFont",
    sfz_path_long_help = "The path to an SFZ SoundFont",

    layer_limit_short_help = "Enable the voice limiter",
    layer_limit_long_help =
        "Limit the number of voices that the synth can create to \
        `Layer Count`",

    layer_count_short_help = "The maximum amount of voices allowed",
    layer_count_long_help =
        "The maximum amount of voices allowed to play per key per channel",

    linear_envelope_short_help = "??????????????",
    linear_envelope_long_help =
        "??????????",

    use_effects_short_help = "Enable the synth's effects",
    use_effects_long_help =
        "Enable the effects that the synthesizer applies to the final audio \
        render. These effects include a limiter to keep the audio from clipping, \
        and a cutoff",
}
