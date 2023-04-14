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

    buffer_ms_short_help = "The size of the event buffer",
    buffer_ms_long_help =
        "The amount of time in milliseconds that events \
        are held in the buffer before they are played. Making this \
        option larger can help MIDI files play in real-time that wouldn't \
        otherwise.",

    sfz_path_short_help = "The path to an SFZ SoundFont",
    sfz_path_long_help =
        "The path to any SFZ soundfont. In audio only mode \
    a soundfont must be passed either via the config file, or
    this command line option. In the GUI you can set this under \
        `Open Synth Settings > SFZ Path`",

    layer_limit_short_help = "Disable the voice limiter",
    layer_limit_long_help =
        "Allow for the synth to create as many voices as \
        needed to play the MIDI file faithfully",

    layer_count_short_help = "The maximum amount of voices allowed",
    layer_count_long_help =
        "The maximum amount of voices allowed to play per key per channel",

    vel_ignore_short_help = "The range of note velocities that the synth will discard",
    vel_ignore_long_help =
        "Two numbers, comma seperated, that represent a range of velocities \
        that the synth will discard, making notes in the range inaudible.",

    fade_out_kill_short_help = "Once a voice is killed, fade it out",
    fade_out_kill_long_help =
        "Once the synthesizer kills one of it's voices, it will fade it \
        out as opposed to simply cutting it off",

    linear_envelope_short_help = "??????????????",
    linear_envelope_long_help =
        "??????????",

    no_effects_short_help = "Disable the synth's effects",
    no_effects_long_help =
        "Disable the effects that the synthesizer applies to the final audio \
        render. These effects include a limiter to keep the audio from clipping, \
        and a cutoff",

    note_speed_short_help = "The speed that the notes travel on-screen",
    note_speed_long_help =
        "The speed at which the notes will move across the screen. This makes \
        the notes physically longer, causing them to move faster on-screen",

    random_colors_short_help = "Make each channel a random color",
    random_colors_long_help =
        "This causes each of the note channels to become a random color",

    key_range_short_help = "The key range of the on-screen piano keyboard",
    key_range_long_help =
        "Two numbers, comma seperated, that describe the range \
        of keys to be shown on the on-screen piano keyboard, the range must \
        be less than 255 and more than 0",

    midi_loading_short_help = "How the MIDI is loaded into `wasabi`",
    midi_loading_long_help =
        "The method in which the MIDI file is loaded into `wasabi`, the \
        two possible options are `ram`, which loads the MIDI file entirely into \
        RAM before beginning playback; and `live` which will read the MIDI file \
        as it's being played back. The latter method is for using with systems \
        with low memory",

    audio_only_short_help = "Don't open a window, just play the MIDI",
    audio_only_long_help =
        "Only initialize the real time MIDI synthesizer, and don't open \
        the `wasabi` window. This will cause a CLI to open which will allow you \
        to control the playback of your MIDI file. You must pass a MIDI file to \
        use this option, and you must have either set `sfz_path` in the config, or \
        passed it via the command line argument",

    bg_color_short_help = "The window background",
    bg_color_long_help =
        "A hex color string describing the background color of the window",

    bar_color_short_help = "The color of the bar just above the piano",
    bar_color_long_help =
        "A hex color string describing the color of the bar just above \
        the on-screen piano keyboard",

    hide_top_panel_short_help = "Hide the top panel",
    hide_top_panel_long_help =
        "Hides the top panel from view when the app opens. It can be un-hidden \
        with Ctrl+F",

    hide_statistics_short_help = "Hide the statistics window",
    hide_statistics_long_help =
        "Hides the statistics window from view when the app opens. It can be \
        un-hidden with Ctrl+G",

    fullscreen_short_help = "Start `wasabi` in fullscreen",
    fullscreen_long_help =
        "Starts `wasabi` in fullscreen mode. `wasabi` will use \
        borderless fullscreen mode on Linux systems running Wayland, \
        and exclusive fullscreen mode for everyone else",

    midi_file_short_help = "The MIDI file to immediately begin playing",
    midi_file_long_help =
        "This MIDI file is played immediately after the app's launch. \
        This argument is required to use the `--audio-only` option",
}
