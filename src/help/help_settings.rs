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
    synth_short_help = "The synthesizer to use",
    synth_long_help =
        "The synthesizer that is used to play the MIDI. \
        This can either be XSynth (recommended) or KDMAPI. KDMAPI \
        only works if you have OmniMidi installed, and are using Windows",

    note_speed_short_help = "The speed that the notes travel on-screen",
    note_speed_long_help =
        "The speed at which the notes will move across the screen. This makes \
        the notes physically longer, causing them to move faster on-screen",

    random_colors_short_help = "Make each channel a random color",
    random_colors_long_help =
        "This causes each of the note channels to become a random color",

    key_range_short_help = "The key range of the on-screen piano keyboard",
    key_range_long_help =
        "The range of keys to be shown on the on-screen piano keyboard, \
        the range must be less than 255 and more than 0",

    midi_loading_short_help = "How the MIDI is loaded into `wasabi`",
    midi_loading_long_help =
        "The method in which the MIDI file is loaded into `wasabi`, the \
        two possible options are `Into RAM`, which loads the MIDI file entirely into \
        RAM before beginning playback; and `Live` which will read the MIDI file \
        as it's being played back. The latter method is for using with systems \
        with low memory",

    bg_color_short_help = "The window background",
    bg_color_long_help =
        "The background color of the window",

    bar_color_short_help = "The color of the bar just above the piano",
    bar_color_long_help =
        "The color of the bar just above the on-screen piano keyboard",
}
